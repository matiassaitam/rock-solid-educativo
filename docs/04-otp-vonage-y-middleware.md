# 04 · OTP, Vonage y middleware

Esta parte explica la solución final de `develop`. Lee primero [routes.rs](../src/routes.rs), después [vonage.rs](../src/vonage.rs) y [token.rs](../src/token.rs). Para arrancar y hacer comprobaciones locales, sigue [05 · Probar y revisar](05-probar-y-revisar.md).

## El recorrido completo

```text
Usuario y contraseña
        ↓
Repositorio → comprobar contraseña
        ↓
Identity Insights → comprobar cambio reciente de SIM
        ↓
Verify → enviar OTP por SMS → devolver identificador
        ↓
Usuario introduce código → Verify comprueba código
        ↓
Crear JWT de sesión → acceder a /application
```

Un OTP es un código de un solo uso. Añade una comprobación de acceso al teléfono después de la contraseña. El control de SIM swap busca cambios recientes de SIM que podrían representar un riesgo; una respuesta favorable no demuestra por sí sola que una persona sea legítima.

En la versión inicial del taller, `/session` entregaba directamente un JWT. En esta versión entrega `verify_request_id`: **todavía falta comprobar el OTP**.

## Qué hace cada endpoint

| Método y ruta | Entrada | Resultado correcto | Acceso |
|---|---|---|---|
| `GET /health` | Ninguna | `200`, `live` y `ready` | Público |
| `POST /session` | `username`, `password` | `200`, `verify_request_id` | Público |
| `POST /session/verification` | `username`, `request_id`, `code` | `201`, `token` | Público |
| `GET /application` | `Authorization: Bearer …` | `200`, mensaje de ejemplo | JWT válido |

Público significa que no exige un JWT previo; los handlers siguen comprobando sus entradas. `/health` devuelve valores fijos: no comprueba la disponibilidad de Vonage.

`create_session` busca al usuario y verifica su contraseña. Si falla, devuelve `401` antes de llamar a Vonage. El teléfono sale del usuario almacenado, no del JSON del login.

Después crea un JWT para hablar con Vonage y ejecuta dos operaciones en orden: `check_sim_swap` y `start_verification`. La segunda devuelve un identificador que el cliente utilizará para validar el código.

## El cliente HTTP compartido

`AppState` contiene el repositorio y un `VonageClient`. Este último encapsula un `reqwest::Client`, creado una vez al construir el router. Reutilizarlo permite aprovechar sus conexiones; clonar el cliente comparte su infraestructura interna.

Cada llamada construye un cuerpo serializable, añade autenticación Bearer y usa `.send().await`. Mientras espera la red, Tokio puede ejecutar otras tareas. `response.json().await` convierte la respuesta JSON en estructuras Rust.

El cliente tiene un timeout de **5 segundos por petición saliente**. `channel_timeout: 600` configura otro concepto: el tiempo del canal de verificación solicitado al proveedor. No es el timeout de la petición HTTP.

## Qué significa fallar de forma cerrada

`check_sim_swap` consulta los últimos siete días. Solo permite seguir si el proveedor responde correctamente, devuelve estado `OK` e indica explícitamente `Some(false)` en `is_swapped`.

Un cambio detectado produce `403`. Un resultado ausente, incompleto, no admitido o fallido impide continuar y normalmente termina en `502` mediante `AppError`. **No saber si existe riesgo no equivale a confirmar que no existe.** Esa decisión se denomina *fail-closed*.

El endpoint de Identity Insights es europeo por defecto; `VONAGE_IDENTITY_INSIGHTS_REGION=us` selecciona el estadounidense. Una configuración regional inválida es un error interno.

## Dos JWT y una limitación del ejemplo

El JWT de Vonage identifica la aplicación ante el proveedor y dura 600 segundos. El JWT de sesión identifica al usuario mediante `sub`, contiene `exp` y se emite para 3600 segundos. Utilizan funciones y claves distintas.

**La asociación entre OTP y usuario es insegura en este ejemplo.** `validate_code` comprueba `request_id` y `code` con Vonage, pero emite el JWT para el `username` enviado por el cliente. El servidor no demuestra que ese usuario sea el propietario del desafío verificado.

La corrección propuesta consiste en guardar un desafío pendiente que vincule identificador, usuario y caducidad; después se obtiene el usuario de ese registro y se consume el desafío. Está explicada en la guía original y advertida en el código, pero **no está implementada**.

## Middleware y protección real

`require_session` se aplica exclusivamente al router de `/application`. Lee la clave pública, extrae el Bearer y valida el JWT con RS256, incluida su expiración según la configuración de `jsonwebtoken`. Solo después llama a `next.run(request)`.

Validar el JWT no implementa permisos por rol. Aquí se descartan los claims y cualquier sesión válida obtiene el mismo mensaje. Para restringir acciones habría que conservar la identidad y comprobar permisos sobre cada operación o recurso.

| Protección | Estado en el código actual |
|---|---|
| Trazas HTTP con `TraceLayer` | Implementada |
| Timeout saliente de `reqwest` | Implementado: 5 segundos |
| Timeout global del servidor | Propuesto en `INSTRUCTIONS.md`; no instalado |
| CORS | Propuesto; no instalado |
| Límites de peticiones e intentos | Propuestos; no instalados |
| Vinculación del desafío al usuario | Advertida; pendiente |

## Orden de prueba manual

Estos comandos son para **Git Bash**, no PowerShell. Son plantillas; sustituye los valores entre `<…>`. No se han ejecutado llamadas a Vonage para preparar esta explicación.

1. Comprueba el servidor y el rechazo sin sesión. Para obtener `401`, la clave pública debe estar correctamente configurada:

```bash
curl -i http://127.0.0.1:8080/health
curl -i http://127.0.0.1:8080/application
```

2. Con configuración de Vonage válida y un teléfono de prueba autorizado, solicita el desafío:

```bash
curl -i http://127.0.0.1:8080/session \
  -H 'Content-Type: application/json' \
  -d '{"username":"<USUARIO>","password":"<CONTRASENA>"}'
```

3. Copia `verify_request_id` como `request_id` y utiliza el código recibido:

```bash
curl -i http://127.0.0.1:8080/session/verification \
  -H 'Content-Type: application/json' \
  -d '{"username":"<USUARIO>","request_id":"<ID>","code":"<CODIGO>"}'
curl -i http://127.0.0.1:8080/application \
  -H 'Authorization: Bearer <TOKEN_RECIBIDO>'
```

Los pasos 2 y 3 generan tráfico a APIs externas; el segundo puede enviar un SMS. Revisa acceso, entorno y costes de tu cuenta. No asumas gratuidad ni uses el teléfono de ejemplo como destinatario real.

## Dos ejercicios resueltos

**1. Identity Insights responde `OK`, pero sin `is_swapped`. ¿Se envía el SMS?** No: `None` produce un error y `?` detiene el handler antes de `start_verification`.

**2. ¿Un JWT válido convierte a su titular en administrador?** No. El middleware comprueba autenticación. No existen roles ni decisiones de autorización por usuario en este endpoint.
