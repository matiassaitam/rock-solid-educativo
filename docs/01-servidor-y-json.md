# 1. Servidor HTTP, Tokio, crates y JSON

[Índice](../GUIA_PRACTICA.md) · [Siguiente: repositorio y estado](02-repositorio-y-estado.md)

## Qué estás construyendo

Un servidor es un programa que permanece esperando peticiones. En este taller escucha en `127.0.0.1:8080`: la dirección de tu propio equipo y el puerto elegido para la API. Una petición contiene un método, una ruta, cabeceras y, a veces, un cuerpo. La respuesta incluye un estado HTTP, cabeceras y un cuerpo.

Ejemplo: `GET /health` pregunta por el estado del servicio. Este taller devuelve `200 OK` con `{"live":true,"ready":true}`.

## Cargo, paquete, crate y módulo

**Cargo** gestiona dependencias y construye el proyecto. Un **paquete** se describe mediante `Cargo.toml` y puede contener uno o varios **crates**. Un crate es una unidad que Rust compila: puede producir una biblioteca reutilizable o un ejecutable.

Aquí `src/main.rs` es la raíz del crate ejecutable. `routes.rs`, `user.rs` y los demás son **módulos dentro de ese crate**, no crates independientes. `mod routes;` incorpora el módulo; `use` permite referirse a sus elementos con nombres más cortos.

Cuando el profesor ejecuta:

```sh
cargo add serde_json
cargo add anyhow
```

añade dependencias a `Cargo.toml`. Las versiones concretas que resuelve Cargo quedan en `Cargo.lock`. `cargo run` compila y ejecuta. `cargo fmt --check` comprueba el formato. En esta copia las dependencias de la solución completa ya están declaradas.

## El papel de Tokio

En [main.rs](../src/main.rs), `#[tokio::main]` prepara el runtime que ejecuta el código asíncrono de `main`. `async` permite que la función suspenda su progreso en operaciones que todavía no han terminado.

```rust
let listener = TcpListener::bind(SERVER_ADDR)
    .await
    .expect("Unable to create listener");
axum::serve(listener, app(repo)).await.unwrap();
```

`TcpListener::bind` crea el punto de escucha. Axum recibe ese listener y el router. Cuando una operación de entrada/salida todavía no está lista, `.await` permite suspender esa tarea y que el runtime avance con otras. No significa que todo el código se ejecute en paralelo ni vuelve asíncrona una operación bloqueante.

`expect` y `unwrap` extraen el resultado correcto o provocan un panic si hay un error. En este arranque, un puerto ocupado provoca que el programa termine. Más adelante puedes sustituirlos por propagación de errores con `?`.

## El papel de Axum

[routes.rs](../src/routes.rs) asocia cada petición con una función:

```rust
.route("/health", get(health))
```

`get(health)` atiende el método GET. La función `health` es un **handler**. En la versión final devuelve:

```rust
async fn health() -> Result<Json<HealthData>, AppError>
```

`Result` representa éxito (`Ok`) o error (`Err`). `Json<HealthData>` indica que el cuerpo correcto se enviará como JSON. `AppError` es el tipo de error de la aplicación; se explica en el capítulo 3.

## Serializar una estructura

```rust
#[derive(Serialize)]
struct HealthData {
    live: bool,
    ready: bool,
}
```

La estructura define datos tipados: dos valores booleanos. `#[derive(Serialize)]` genera la implementación que permite serializarlos. El handler construye el valor y lo envuelve:

```rust
Ok(Json(HealthData {
    live: true,
    ready: true,
}))
```

Axum utiliza la serialización JSON y añade `Content-Type: application/json`. El cliente recibe un objeto JSON, no una cadena que contenga JSON escapado. No necesitas llamar antes a `serde_json::to_string` dentro de `Json`.

**Serializar** convierte datos Rust en un formato que puedes transmitir. **Deserializar** hace el recorrido inverso. Por eso `SessionRequest` deriva `Deserialize`: el servidor necesita leer el usuario y la contraseña que llegan en el cuerpo JSON.

## Arranque actual frente a la primera etapa

En `milestone/01-basic-server` bastaba el servidor y `/health`. La versión completa primero carga `.env`, configura `tracing` y crea un usuario en memoria. Después arranca Axum. Aunque vayas a consultar solo `/health`, esta versión exige que exista `.env`; sigue [los pasos de prueba](05-probar-y-revisar.md).

## Comprueba que lo entiendes

1. ¿Qué cabecera permite al cliente reconocer el formato de la respuesta? **Respuesta:** `Content-Type: application/json`.
2. Como ejercicio, añade a `HealthData` un campo `service: &'static str` y dale el valor `"rock-solid"` en el handler. ¿Qué cambia? **Respuesta:** el JSON incorpora `"service":"rock-solid"`; también debes inicializar ese nuevo campo para compilar.

Referencia de la primera etapa: [milestone/01-basic-server](https://github.com/jdortiz/rock-solid/tree/milestone/01-basic-server). Las modificaciones del ejercicio son opcionales y no están aplicadas a la solución del profesor.
