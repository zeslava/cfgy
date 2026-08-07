pub mod adapters;
pub mod application;
pub mod config;
pub mod ports;

use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::server::adapters::api::create_router;
use crate::server::adapters::api::handlers::AppState;
use crate::server::adapters::crypto::AesCryptoService;
use crate::server::adapters::persistence::PostgresUserRepository;
use crate::server::application::{
    auth::AuthService, parameters::ParameterService, projects::ProjectService,
};
use crate::server::ports::{ExternalIdentityRepository, UserRepository};

pub async fn run_server() -> anyhow::Result<()> {
    use std::io::Write;
    eprintln!("[1/7] Инициализация логирования...");

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_default();
    if log_format == "json" {
        use tracing_subscriber::EnvFilter;
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .init();
        eprintln!("[1/7] ✓ Логирование инициализировано (JSON формат)");
    } else {
        use tracing_subscriber::EnvFilter;
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .init();
        eprintln!("[1/7] ✓ Логирование инициализировано (человекочитаемый формат)");
    }
    let _ = std::io::stderr().flush();

    eprintln!("[2/7] Загрузка конфигурации...");
    let config = match config::Config::load() {
        Ok(cfg) => {
            eprintln!("[2/7] ✓ Конфигурация загружена из переменных окружения");
            cfg
        }
        Err(e) => {
            eprintln!(
                "[2/7] ⚠ Ошибка загрузки конфигурации: {}. Используются значения по умолчанию.",
                e
            );
            config::Config::default()
        }
    };

    eprintln!("[3/7] Параметры сервера:");
    eprintln!("  - Host: {}", config.server.host);
    eprintln!("  - Port: {}", config.server.port);
    eprintln!("  - Database URL: {}", config.database.url);
    eprintln!("  - Max connections: {}", config.database.max_connections);

    tracing::info!(
        "CFGY Server starting on {}:{}",
        config.server.host,
        config.server.port
    );
    tracing::info!("Database URL: {}", config.database.url);

    eprintln!("[4/7] Подключение к базе данных...");
    eprintln!("  - URL: {}", config.database.url);
    let pool = match PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .max_lifetime(std::time::Duration::from_secs(30 * 60))
        .idle_timeout(std::time::Duration::from_secs(10 * 60))
        .test_before_acquire(true)
        .connect(&config.database.url)
        .await
    {
        Ok(p) => {
            eprintln!("[4/7] ✓ Подключение к базе данных установлено");
            p
        }
        Err(e) => {
            eprintln!("[4/7] ✗ Ошибка подключения к базе данных: {}", e);
            eprintln!("[4/7] ✗ Детали: {:?}", e);
            return Err(anyhow::anyhow!("Failed to connect to database: {}", e));
        }
    };

    tracing::info!("Connected to database successfully");

    eprintln!("[4/7] Применение миграций...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;
    eprintln!("[4/7] ✓ Миграции применены");

    eprintln!("[4/7] Проверка подключения к базе данных...");
    let result: (i64,) = match sqlx::query_as::<_, (i64,)>("SELECT $1")
        .bind(1_i64)
        .fetch_one(&pool)
        .await
    {
        Ok(r) => {
            eprintln!("[4/7] ✓ Тестовый запрос выполнен успешно: {}", r.0);
            r
        }
        Err(e) => {
            eprintln!("[4/7] ✗ Ошибка выполнения тестового запроса: {}", e);
            return Err(anyhow::anyhow!("Database query test failed: {}", e));
        }
    };

    tracing::info!("Database query test result: {}", result.0);

    eprintln!("[5/7] Инициализация репозиториев...");
    let repositories = PostgresUserRepository::new(pool.clone());
    eprintln!("[5/7] ✓ Репозитории инициализированы");

    let user_repo: Arc<dyn UserRepository> = Arc::new(repositories.clone());
    let external_identity_repo: Arc<dyn ExternalIdentityRepository> =
        Arc::new(repositories.clone());

    eprintln!("[5/7] Инициализация криптографического сервиса...");
    let master_key_bytes = config.crypto.master_key.as_bytes();
    let crypto_service = match AesCryptoService::new(master_key_bytes) {
        Ok(service) => {
            eprintln!("[5/7] ✓ Криптографический сервис инициализирован");
            service
        }
        Err(e) => {
            eprintln!(
                "[5/7] ✗ Ошибка инициализации криптографического сервиса: {}",
                e
            );
            return Err(anyhow::anyhow!(
                "Failed to initialize crypto service: {}",
                e
            ));
        }
    };

    eprintln!("[6/7] Инициализация сервисов приложения...");
    let project_service = Arc::new(ProjectService::new(
        repositories.clone(),
        repositories.clone(),
        repositories.clone(),
        crypto_service.clone(),
    ));
    eprintln!("[6/7] ✓ ProjectService инициализирован");

    let parameter_service = Arc::new(ParameterService::new(
        repositories.clone(),
        repositories.clone(),
        repositories.clone(),
        crypto_service,
    ));
    eprintln!("[6/7] ✓ ParameterService инициализирован");

    let auth_service = Arc::new(AuthService::new(repositories.clone(), repositories));
    eprintln!("[6/7] ✓ AuthService инициализирован");

    eprintln!("[6/7] Создание состояния приложения...");
    let app_state = AppState {
        project_service,
        parameter_service,
        auth_service,
        user_repo,
        external_identity_repo,
        db_pool: pool.clone(),
    };
    eprintln!("[6/7] ✓ Состояние приложения создано");

    eprintln!("[6/7] Создание роутера...");
    let app = create_router(app_state);
    eprintln!("[6/7] ✓ Роутер создан");

    eprintln!("[7/7] Запуск HTTP сервера...");
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let server_url = format!("http://localhost:{}", config.server.port);

    eprintln!("\n{}", "=".repeat(60));
    eprintln!("CFGY Server запущен!");
    eprintln!("Адрес сервера: {}", server_url);
    eprintln!("Локальный адрес: http://127.0.0.1:{}", config.server.port);
    eprintln!("Сетевой адрес: http://0.0.0.0:{}", config.server.port);
    eprintln!("{}\n", "=".repeat(60));

    tracing::info!("Listening on {}", addr);

    eprintln!("[7/7] Привязка к адресу {}...", addr);
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => {
            eprintln!("[7/7] ✓ Сервер привязан к адресу {}", addr);
            l
        }
        Err(e) => {
            eprintln!("[7/7] ✗ Ошибка привязки к адресу {}: {}", addr, e);
            return Err(anyhow::anyhow!("Failed to bind to address {}: {}", addr, e));
        }
    };

    eprintln!("[7/7] ✓ Сервер готов принимать подключения");
    eprintln!("[7/7] Ожидание запросов...\n");

    let shutdown = async {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
        tracing::info!("Получен сигнал завершения, начинается graceful shutdown...");
        eprintln!("\n[shutdown] Получен сигнал завершения, начинается graceful shutdown...");
    };

    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
    {
        eprintln!("[7/7] ✗ Ошибка при работе сервера: {}", e);
        eprintln!("[7/7] ✗ Детали: {:?}", e);
        return Err(anyhow::anyhow!("Server error: {}", e));
    }

    tracing::info!("Сервер завершил работу");
    eprintln!("[shutdown] Сервер успешно завершил работу");

    Ok(())
}
