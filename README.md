# rock-solid-educativo

## Guía educativa en español

Copia educativa basada en la rama `develop` de [jdortiz/rock-solid](https://github.com/jdortiz/rock-solid), commit [`48d6073`](https://github.com/jdortiz/rock-solid/commit/48d6073). Conserva la solución del taller de Jorge D. Ortiz Fuentes y añade explicaciones en español. En este repositorio, `main` contiene la solución y la documentación guardadas en commits.

Empieza en [GUIA_PRACTICA.md](./GUIA_PRACTICA.md): explica cada archivo, los conceptos de Rust, el flujo de autenticación, las etapas del taller y cómo probar y revisar esta solución.

Es material de práctica, no una implementación preparada para producción: el desafío OTP no está vinculado al usuario en el servidor, hay credenciales de demostración y faltan límites de intentos. La licencia y atribución del proyecto original se conservan al final de este README.

## This repo

Build a rock-solid HTTP service in Rust from scratch with Axum. The workshop covers the four As of security:
authentication, authorization, accounting, and auditing. It uses JWTs, middleware, structured tracing, and unified error
handling.

This repository contains the Rust code for the presentation [From Scratch to Rock-Solid: Secure HTTP Services in Rust with
Axum](./INSTRUCTIONS.md).

## Open the repository in a Codespace

1. Open this repository on GitHub in **Chrome**.
2. Select **Code**, open the **Codespaces** tab, and create a Codespace for the repository.
3. Chrome is recommended for this workshop; **Safari is not supported**.
4. Wait for the container setup to finish. The setup installs the required tools, fetches the Rust dependencies, and runs
   `cargo check`. Start the service only after the setup has completed.

The development container forwards port `8080` for the API. Configure the application before starting it.

## Configuration

The application requires a `.env` file in the repository root. Do not commit it. The repository ignores `.env` files, but
secrets should still be handled carefully.

Generate the JWT key pair first:

```console
./generate-env.sh
```

Then add the Vonage settings to the generated `.env` file. The file uses `KEY="value"` entries, for example:

```dotenv
JWT_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----
<generated-private-key>
-----END PRIVATE KEY-----"
JWT_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----
<generated-public-key>
-----END PUBLIC KEY-----"
VONAGE_APP_ID="<your-application-id>"
VONAGE_SECRET_KEY="-----BEGIN PRIVATE KEY-----
<contents-of-private.key>
-----END PRIVATE KEY-----"
VERIFY_PHONE="<phone-number-with-country-code>"
VONAGE_IDENTITY_INSIGHTS_REGION="eu"
```

`JWT_PRIVATE_KEY` and `JWT_PUBLIC_KEY` are created by `generate-env.sh`. Obtain `VONAGE_APP_ID` and
`VONAGE_SECRET_KEY` from the Vonage application, and set `VERIFY_PHONE` to the permitted test number in the format
expected by the API. `VONAGE_IDENTITY_INSIGHTS_REGION` is optional; use `eu` or `us` when the workshop code supports
regional selection.

Keep private keys, API credentials, and phone numbers out of commits, screenshots, and logs.

## Run and test

Start the service from the Codespace terminal:

```console
cargo run
```

When the service is running, open the forwarded URL from the **Ports** panel. You can also test it inside the Codespace:

```console
curl -i http://127.0.0.1:8080/health
```

## Workshop instructions

Follow the complete, step-by-step workshop in [INSTRUCTIONS.md](./INSTRUCTIONS.md).

## Workshop checkpoints

In the [original repository](https://github.com/jdortiz/rock-solid), `main` is the starting point and `develop` contains the accumulated solution. In this educational copy, `main` contains the solution and the Spanish study material.

The tags below belong to the original workshop; they are available locally only if you have imported them. To import the original tags and inspect a checkpoint:

```console
git fetch https://github.com/jdortiz/rock-solid.git --tags
git switch --detach milestone/05-error-handling
```

Original workshop checkpoints:

| Tag | Topic |
| --- | --- |
| `milestone/00-start` | Starting point |
| `milestone/01-basic-server` | Basic HTTP server |
| `milestone/02-user-repository` | User repository |
| `milestone/03-session-jwt` | Session JWT |
| `milestone/04-tracing` | Logging and tracing |
| `milestone/05-error-handling` | Unified error handling |
| `milestone/06-verify-start` | Start Verify authentication |
| `milestone/07-verify-check` | Verify the OTP |
| `milestone/08-identity-insights` | Identity Insights risk check |
| `milestone/09-protected-endpoint` | Protected endpoint |
| `milestone/10-middleware` | JWT middleware |
| `milestone/complete` | Complete workshop |

`git switch --detach` lets you inspect or run an imported checkpoint without changing `main`. With a clean working tree, return to the published educational solution with:

```console
git switch main
```

Once the original tag has been imported, create a branch if you want to make changes from that checkpoint:

```console
git switch -c my-work milestone/05-error-handling
```

## License

This project is licensed under the terms of the [Apache License 2.0](./LICENSE.txt).

## Author

Jorge D. Ortiz Fuentes, 2026

## Resources

To learn more about Jorge's work, see:

- [🖋️ Jorge's Blog](https://jorgeortiz.dev/)
- [🙋🏻‍♂️ Jorge's LinkedIn](https://www.linkedin.com/in/jorgeortiz/)
- [🙋🏻‍♂️ Jorge's Reddit](https://www.reddit.com/user/jorgedortiz/)
- [🙋🏻‍♂️ Jorge's Bluesky](https://bsky.app/profile/jdortiz.bsky.social)
- [🙋🏻‍♂️ Jorge's Mastodon](https://fosstodon.org/@jdortiz)
- [🙋🏻‍♂️ Jorge's X](https://x.com/jdortiz)
