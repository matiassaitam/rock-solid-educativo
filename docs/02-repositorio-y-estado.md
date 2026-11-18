# Repositorio, usuarios y estado compartido

Esta parte explica cómo el servidor encuentra usuarios y comparte sus dependencias. Lee primero [repository.rs](../src/repository.rs), después [user.rs](../src/user.rs), [main.rs](../src/main.rs) y [routes.rs](../src/routes.rs). El punto de partida de esta lección está en [milestone/02-user-repository](https://github.com/jdortiz/rock-solid/tree/milestone/02-user-repository); `AppState` pertenece a la solución completa que estás revisando.

## Qué significa Repository

Un repositorio Git guarda código e historial. Aquí, el patrón **Repository** define operaciones para guardar y recuperar datos de la aplicación. El trait es el contrato; `InMemoryRepository` es una implementación que conserva los usuarios en memoria.

Estas son sus dos operaciones, copiadas del código:

```rust
fn add(&mut self, item: T);
fn find_by_id(&self, id: &T::Id) -> Option<&T>;
```

`add` necesita acceso mutable y recibe la propiedad del objeto. `find_by_id` consulta sin modificar: devuelve `Some(&usuario)` si lo encuentra o `None` si no existe. La referencia prestada sigue dependiendo del repositorio; no es una copia independiente.

## Identifiable, tipos asociados y HashMap

`Identifiable` exige que cada objeto tenga un identificador:

```rust
type Id: Clone + Eq + Hash + Send + Sync;

fn id(&self) -> &Self::Id;
```

`Id` es un **tipo asociado**: cada implementación concreta el tipo de su identificador. En `User`, `type Id = String` y `id()` devuelve una referencia a `username`. Por tanto, `T::Id` significa «el tipo de identificador del objeto `T`».

El almacenamiento es `HashMap<T::Id, T>`: un mapa que relaciona identificadores con objetos. `Eq` permite comparar claves, `Hash` permite calcular dónde buscarlas y `Clone` permite copiar el identificador antes de guardar el objeto. Al insertar otra vez la misma clave, `HashMap::insert` reemplaza el valor anterior. Al cerrar el proceso, estos datos desaparecen.

## Cómo leer Arc y dyn

En `main` aparece:

```rust
let repo: Arc<dyn Repository<User>> = Arc::new(seed_users());
```

`Arc` comparte la propiedad de una instancia mediante un contador de referencias atómico. Clonarlo crea otro propietario del mismo repositorio; no duplica usuarios. Cuando desaparece el último propietario, se libera el objeto.

`dyn Repository<User>` permite utilizar una implementación concreta a través del trait. Rust resuelve las llamadas a sus métodos mediante despacho dinámico. Aquí se construye un `InMemoryRepository<User>`; `dyn` no cambia automáticamente esa implementación durante la ejecución.

El contrato declara `Repository<T>: Send + Sync`. `Send` permite transferir propiedad entre hilos y `Sync` permite compartir referencias entre hilos. **`Arc` no vuelve seguro cualquier contenido ni sincroniza modificaciones.** Estas garantías también dependen del tipo contenido.

En esta práctica se añaden usuarios antes de crear el `Arc`; después se consulta el repositorio. Para modificarlo concurrentemente haría falta otro diseño, por ejemplo con `Mutex` o `RwLock` y un contrato apropiado. No basta con añadir `Arc`.

## Inversión, inyección y estado de Axum

La **inversión de dependencias** consiste en que el código que necesita usuarios dependa del contrato `Repository<User>`, en lugar de los detalles del `HashMap`.

La **inyección de dependencias** es cómo se entrega esa dependencia: `main` construye el repositorio y se lo pasa a `app(repo)`.

En la solución completa, `AppState` agrupa las dependencias:

```rust
#[derive(Clone)]
struct AppState {
    repo: Arc<dyn Repository<User>>,
    vonage: VonageClient,
}
```

`.with_state(state)` registra ese estado en el router. El parámetro `State(state): State<AppState>` permite obtenerlo en un handler. `Clone` genera la operación de clonación del estado; al clonar su `Arc`, las peticiones siguen compartiendo el repositorio. No exige que `User` sea `Clone`.

## Contraseñas y límites del ejemplo

`User::new` guarda un hash bcrypt en el campo privado `hashed_password`. El hash incluye la información necesaria para verificar, como la sal y el coste. No se descifra para recuperar la contraseña: `verify_password` comprueba la entrada mediante `bcrypt::verify` y devuelve `false` si la verificación falla o produce un error.

La contraseña de demostración aparece en `seed_users`; no representa un sistema real de registro. Además, bcrypt realiza trabajo síncrono costoso: con carga real convendría sacarlo de los hilos asíncronos que atienden peticiones.

Cambiar a PostgreSQL tampoco consiste simplemente en sustituir una clase: el contrato actual es síncrono y devuelve referencias a objetos ya almacenados. Una consulta de red necesita esperar y puede fallar; habría que diseñar operaciones asíncronas y resultados con errores, normalmente devolviendo datos propios, y conservar la compatibilidad con `dyn` si se desea seguir utilizándolo.

## Mapa para recorrer el código

| Función | Responsabilidad |
| --- | --- |
| `InMemoryRepository::new` | Crear el mapa vacío. |
| `add` / `find_by_id` | Guardar y localizar objetos. |
| `User::new` / `verify_password` | Crear el hash y verificar contraseñas. |
| `seed_users` | Preparar el usuario de demostración. |
| `app` / `create_session` | Conectar el estado y consultar al usuario durante el login. |

## Dos ejercicios

1. ¿Qué ocurre si `seed_users` añade dos usuarios con el mismo `username`? **Respuesta:** queda el último, porque el identificador es la clave del mapa.
2. ¿Clonar `AppState` permite modificar los usuarios mediante `repo.add(...)`? **Respuesta:** no. Comparte el mismo repositorio mediante `Arc`; `add` exige acceso mutable exclusivo que ese acceso compartido no proporciona.
