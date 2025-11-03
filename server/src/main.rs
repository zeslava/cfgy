mod config;
mod ports;
mod adapters;
mod application;

use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::adapters::api::create_router;
use crate::adapters::crypto::AesCryptoService;
use crate::adapters::persistence::PostgresRepositories;
use crate::application::{auth::AuthService, projects::ProjectService, secrets::SecretService};
use crate::adapters::api::handlers::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::Config::load().unwrap_or_default();

    tracing::info!(
        "CFGY Server starting on {}:{}",
        config.server.host,
        config.server.port
    );
    tracing::info!("Database URL: {}", config.database.url);

    // Инициализация пула подключений к PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&config.database.url)
        .await?;

    tracing::info!("Connected to database successfully");

    // Проверка подключения через простой запрос
    let result: (i64,) = sqlx::query_as("SELECT $1")
        .bind(1_i64)
        .fetch_one(&pool)
        .await?;

    tracing::info!("Database query test result: {}", result.0);

    // Инициализация репозиториев
    let repositories = PostgresRepositories::new(pool);

    // Инициализация криптографического сервиса
    let master_key_bytes = config.crypto.master_key.as_bytes();
    let crypto_service = AesCryptoService::new(master_key_bytes)?;

    // Инициализация сервисов приложения
    let project_service = Arc::new(ProjectService::new(
        repositories.clone(),
        repositories.clone(),
        repositories.clone(),
        crypto_service.clone(),
    ));

    let secret_service = Arc::new(SecretService::new(
        repositories.clone(),
        repositories.clone(),
        repositories.clone(),
        crypto_service,
    ));

    let auth_service = Arc::new(AuthService::new(repositories.clone(), repositories));

    // Создание состояния приложения
    let app_state = AppState {
        project_service,
        secret_service,
        auth_service,
    };

    // Создание роутера
    let app = create_router(app_state);

    // Запуск сервера
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
