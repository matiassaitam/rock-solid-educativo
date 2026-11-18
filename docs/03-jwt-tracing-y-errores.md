# 03. JWT, tracing y errores

Esta lección conecta la autenticación con las respuestas HTTP. Lee [routes.rs](../src/routes.rs), [token.rs](../src/token.rs) y [app_error.rs](../src/app_error.rs) junto a las secciones «Create a session and issue a JWT», «Add logging and tracing» e «Implement unified error handling» de [INSTRUCTIONS.md](../INSTRUCTIONS.md).

## 1. Del usuario y contraseña al token

En las etapas 03–05, el cliente envía `username` y `password` a `POST /session`. El handler busca al usuario en el repositorio y comprueba la contraseña con `verify_password`. Si las credenciales son válidas, genera un JWT de sesión. Una contraseña incorrecta y un usuario inexistente producen el mismo mensaje externo: `Invalid credentials`.

El código final amplía ese recorrido: `/session` comprueba las credenciales, consulta el riesgo de SIM swap y solicita un OTP. Devuelve `verify_request_id`, **todavía no el JWT de sesión**. Después, `POST /session/verification` valida el código con Vonage y devuelve el token con estado **201 Created**.

Hay una limitación explícita del taller: este segundo endpoint acepta `username` del cliente. Aunque valide el OTP, no comprueba aquí que ese nombre corresponda al desafío. Una implementación completa debe guardar en el servidor la relación entre usuario, desafío y vencimiento, y consumirla al verificarlo.

## 2. Cómo entra y sale JSON

`serde` define los traits de serialización; `serde_json` implementa el formato JSON. `derive` genera implementaciones para tus estructuras:

```rust
#[derive(Deserialize)]
struct SessionRequest {
    username: String,
    password: String,
}
```

`Deserialize` convierte los datos entrantes en una estructura Rust. `Serialize` permite convertir una estructura en datos de salida. Axum utiliza ambos mediante `Json<T>`: extrae JSON en una petición y construye una respuesta JSON con su cabecera correspondiente. `SessionClaims` necesita ambos traits porque el servicio escribe y lee sus claims; `VonageClaims` solamente deriva `Serialize` porque aquí solo se emite ese token.

## 3. Qué hace cada función de `token.rs`

Un JWT contiene cabecera, payload y firma. El payload guarda **claims**, afirmaciones como quién es el usuario y cuándo vence la sesión. En este proyecto la firma usa **RS256**: RSA con SHA-256. La clave privada firma y la pública permite verificar. **Firmar no cifra el contenido**: no guardes contraseñas ni secretos en los claims.

| Función | Responsabilidad |
| --- | --- |
| `get_encoding_key` | Lee `JWT_PRIVATE_KEY` del entorno e interpreta su contenido como una clave RSA PEM. |
| `get_decoding_key` | Lee `JWT_PUBLIC_KEY` e interpreta la clave pública PEM. |
| `produce_session_token` | Firma `sub`, el nombre de usuario, y `exp`, el vencimiento calculado a una hora. |
| `verify_session_token` | Extrae `Authorization: Bearer ...`, decodifica con validación RS256 y rechaza cabeceras o tokens inválidos. |
| `produce_vonage_token` | Firma otro JWT para las APIs externas: usa `VONAGE_SECRET_KEY`, `VONAGE_APP_ID`, `iat`, `exp` a diez minutos y un `jti` UUID nuevo. |
| `get_now_in_secs` | Obtiene los segundos transcurridos desde el inicio de Unix mediante el reloj del sistema. |

Las variables de claves contienen **el PEM**, no una ruta. Los tokens de sesión y de Vonage tienen destinatarios y propósitos diferentes.

La validación de sesión incluye firma y expiración mediante `Validation::new(Algorithm::RS256)`. Devuelve `Result<(), AppError>`: descarta los claims tras verificarlos. El middleware `require_session` continúa hacia `/application` solo si la validación funciona. Este código autentica; todavía no decide permisos por rol o por recurso.

## 4. `anyhow` dentro, `AppError` en HTTP

`cargo add anyhow` añade una dependencia para propagar errores de distintos orígenes con `anyhow::Result<T>`. Resulta útil, por ejemplo, cuando leer una variable o interpretar una clave puede fallar.

`AppError` aporta categorías propias. `thiserror` implementa el comportamiento de error y `#[from] anyhow::Error` permite convertir errores internos automáticamente. Así, `?` devuelve el error al llamador y realiza la conversión disponible:

```rust
let encoding_key = get_encoding_key()?;
let token = produce_session_token(&encoding_key, &payload.username)?;
```

`IntoResponse` convierte cada categoría en una respuesta HTTP:

| Variante | Estado |
| --- | --- |
| `BadRequest` | 400 |
| `Unauthorized` | 401 |
| `Forbidden` | 403 |
| `NotFound` | 404 |
| `Internal` | 500 |
| `ExternalApiError` | 502 |

El cuerpo mantiene la forma `{"error":{"message":"...","type":"..."}}`. Para `Internal`, registra el error real y devuelve `An unexpected error occurred`, ocultando detalles al cliente. La conversión desde `reqwest::Error` produce un error externo con mensaje genérico.

Los handlers reciben `Result<Json<...>, JsonRejection>`. Esto captura rechazos del extractor; `payload?` los transforma mediante `From<JsonRejection>` en `BadRequest`. Un JSON mal formado, con campos incompatibles o una cabecera de contenido incorrecta sigue así el mismo formato de error.

## 5. Observar sin revelar secretos

[main.rs](../src/main.rs) configura `tracing_subscriber` y su filtro `RUST_LOG`. En PowerShell:

```powershell
$env:RUST_LOG = "rock_solid=debug,tower_http=info"
cargo run
```

`TraceLayer` registra el recorrido HTTP. `debug!`, `warn!` y `error!` añaden eventos y campos estructurados. El código registra nombres de usuario; decide qué información necesitas conservar. No añadas contraseñas, OTP, claves privadas o tokens completos a los logs. Activar todas las cabeceras expondría potencialmente `Authorization`.

## 6. Dos ejercicios con respuesta

1. **¿Una firma JWT válida permite leer el payload?** Sí. La firma detecta alteraciones y permite verificar su origen; no proporciona confidencialidad.
2. **¿Qué devuelve el flujo final después de una contraseña correcta?** Si también pasan las consultas externas, `/session` devuelve un identificador de verificación. El JWT llega tras validar el OTP. Si falta una clave requerida, su error se transforma en `Internal`, con respuesta 500 genérica.
