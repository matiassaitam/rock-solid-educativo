# 5. Ejecutar, probar y revisar

[Índice](../GUIA_PRACTICA.md) · [Anterior: OTP y middleware](04-otp-vonage-y-middleware.md)

## Qué versión tienes delante

El código procede de la solución acumulada de la rama `develop` del [repositorio original](https://github.com/jdortiz/rock-solid), commit [`48d6073`](https://github.com/jdortiz/rock-solid/commit/48d6073). En esta copia educativa, `main` contiene la solución y las explicaciones de `GUIA_PRACTICA.md` y `docs/`, guardadas en commits.

Después de clonar este repositorio, **Source Control → Changes** estará vacío mientras no modifiques archivos. La solución publicada se revisa abriendo el código, consultando el historial o comparándola con las etiquetas originales si las has importado.

## 1. Preparar Windows

Si usas Rust con el toolchain MSVC en Windows, necesitas también las herramientas de compilación de C++. Un error `link.exe not found` indica que falta el enlazador o que el entorno no lo encuentra.

Abre **Visual Studio Installer → Visual Studio Community 2022 → Modificar**. Añade **Desarrollo para el escritorio con C++**, con las herramientas MSVC x64/x86 y un Windows SDK. Después vuelve a abrir VS Code y el terminal. Consulta los [requisitos oficiales de Rust](https://rust-lang.github.io/rustup/installation/windows-msvc.html).

El repositorio también incluye un dev container para Codespaces. Un Codespace de esta copia en `main` abre la solución educativa. Si trabajas en el repositorio original, selecciona la rama o etiqueta de la lección que quieras estudiar.

## 2. Abrir el proyecto

En VS Code, usa **File → Open Folder** y selecciona la carpeta clonada, normalmente `rock-solid-educativo`, que contiene `GUIA_PRACTICA.md`. Abre un terminal PowerShell dentro de esa carpeta:

```powershell
cargo --version
rustc --version
```

Si el terminal no encuentra Cargo y ya tienes Rust instalado en la ubicación habitual:

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
```

## 3. Arrancar para las pruebas locales

La versión final llama a `dotenvy::dotenv().expect(...)` al arrancar. Para probar únicamente `/health` y entradas incorrectas, crea un `.env` vacío solo si aún no existe:

```powershell
if (-not (Test-Path -LiteralPath '.env')) {
    New-Item -ItemType File -Path '.env' | Out-Null
}
$env:RUST_LOG = 'info'
cargo run --locked
```

Este `.env` vacío no prepara JWT ni Vonage. Solo permite iniciar estas pruebas locales. Si ya tienes configuración, el comando la conserva. Mantén este terminal abierto mientras el servidor escucha en `127.0.0.1:8080`.

Si reaparece `link.exe not found`, falta completar el paso 1. Si aparece un error de descarga o TLS, revisa ese error antes de continuar: el servicio todavía no está arrancado. No desactives la validación de certificados.

## 4. Probar sin enviar SMS

Abre un segundo terminal PowerShell. Estas peticiones no necesitan credenciales de Vonage:

```powershell
curl.exe -i http://127.0.0.1:8080/health
```

Resultado esperado: `200 OK`, `Content-Type: application/json` y `{"live":true,"ready":true}`. Los dos valores son constantes del ejemplo, no un diagnóstico real de las dependencias.

```powershell
curl.exe -i http://127.0.0.1:8080/no-existe
curl.exe -i -X POST http://127.0.0.1:8080/health
```

Resultados esperados: `404 Not Found` y `405 Method Not Allowed` respectivamente.

Envía JSON mal formado:

```powershell
'{' | curl.exe -i http://127.0.0.1:8080/session -H 'Content-Type: application/json' --data-binary '@-'
```

Resultado esperado: `400 Bad Request`, convertido desde `JsonRejection` por `AppError`.

Envía un usuario inexistente:

```powershell
'{"username":"usuario-inexistente","password":"incorrecta"}' | curl.exe -i http://127.0.0.1:8080/session -H 'Content-Type: application/json' --data-binary '@-'
```

Resultado esperado: `401 Unauthorized`. El handler rechaza al usuario antes de crear un token o llamar a Vonage. El patrón de tubería evita problemas al pasar las comillas de JSON desde PowerShell a `curl.exe`.

## 5. Preparar la autenticación completa

Sigue [README.md](../README.md) para las variables JWT y Vonage. `generate-env.sh` usa **zsh** y OpenSSL; no es un script PowerShell. Ejecutarlo escribe o reemplaza `.env`, por lo que conserva tu configuración antes de usarlo. Git Bash por sí solo no garantiza tener zsh.

Las claves de JWT de sesión y la clave privada de la aplicación Vonage tienen funciones diferentes. Usa tu número autorizado de pruebas. El login con credenciales válidas puede consultar APIs y enviar un SMS; revisa las condiciones de tu cuenta antes de probarlo.

El [capítulo 4](04-otp-vonage-y-middleware.md) explica los pasos `/session` → `/session/verification` → `/application`. Para esperar `401` en `/application` sin token, primero configura correctamente la clave pública: el middleware la carga antes de comprobar la cabecera y, si falta, devuelve un error interno.

## 6. Leer Source Control

Abre `src/main.rs` para ver el arranque y `src/routes.rs` para seguir el recorrido de una petición. Después lee `repository.rs`, `user.rs`, `token.rs`, `app_error.rs` y `vonage.rs` junto a sus capítulos.

La solución y las explicaciones están guardadas en commits de `main`. **Source Control → Changes** muestra tus modificaciones posteriores, no todos los archivos ya publicados. Puedes consultar el historial desde el terminal de VS Code:

```sh
git log --oneline --max-count=15
git show HEAD --stat
```

Las referencias `milestone/...` pertenecen al repositorio original. Los siguientes comandos funcionan si has importado sus etiquetas como explica [README.md](../README.md#workshop-checkpoints):

```sh
git diff milestone/00-start HEAD -- src Cargo.toml
git show milestone/01-basic-server:src/routes.rs
git diff milestone/01-basic-server milestone/02-user-repository -- src
```

El primer comando compara el inicio del taller con la solución publicada. El segundo muestra un archivo de una etapa; el tercero muestra el avance entre dos lecciones. No uses `main` como referencia del inicio en esta copia: aquí contiene la solución.

`git show` y `git diff` permiten leer las etapas sin alterar el trabajo. Si modificas código para practicar, abre **Source Control → Changes** y pulsa el archivo para revisar tu diff antes de hacer un commit.

## 7. Comprobaciones y cierre

```sh
cargo fmt --check
cargo check --locked
```

El primer comando verifica formato; el segundo intenta comprobar la compilación. Ninguno sustituye las peticiones HTTP. El repositorio del profesor no incluye una suite de tests Rust: un `cargo test` que informa de cero tests no demuestra que funcione `/health`.

Para detener el servidor, pulsa **Ctrl+C** en su terminal. Si has modificado código, vuelve a ejecutar `cargo run --locked`: este proyecto no incorpora recarga automática.
