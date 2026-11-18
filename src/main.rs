use std::sync::Arc;

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::{
    repository::{InMemoryRepository, Repository},
    routes::app,
    user::User,
};

mod app_error;
mod repository;
mod routes;
mod token;
mod user;
mod vonage;

#[tokio::main]
async fn main() {
    const SERVER_ADDR: &str = "127.0.0.1:8080";
    dotenvy::dotenv().expect("Unable to load configuration from .env");
    tracing_subscriber::fmt()
        // .json()
        // .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let repo: Arc<dyn Repository<User>> = Arc::new(seed_users());

    info!(addr = SERVER_ADDR, "Launching server");
    let listener = TcpListener::bind(SERVER_ADDR)
        .await
        .expect("Unable to create listener");
    axum::serve(listener, app(repo)).await.unwrap();
}

/// Create a user repository for demo purposes.

fn seed_users() -> InMemoryRepository<User> {
    let mut users = InMemoryRepository::new();
    users.add(User::new(
        "alice",
        "secret123",
        std::env::var("VERIFY_PHONE").unwrap_or("34600123456".to_string()),
    ));

    users
}
