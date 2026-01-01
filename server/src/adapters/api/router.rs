use axum::{
    Router,
    body::Body,
    http::{Method, StatusCode, header},
    response::Response,
    routing::{delete, get, patch, post, put},
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use crate::adapters::api::handlers::AppState;

/// Получить путь к index.html, проверяя несколько возможных расположений
/// Все проверки выполняются во время выполнения, чтобы работать независимо от места запуска
fn get_index_path() -> std::path::PathBuf {
    // 1. В Docker контейнере файлы всегда в /app/web/dist
    let docker_path = std::path::Path::new("/app/web/dist/index.html");
    if docker_path.exists() {
        return docker_path.to_path_buf();
    }

    // 2. Относительно текущей рабочей директории (если запускаем из корня проекта)
    let path_from_cwd = std::path::Path::new("web/dist/index.html");
    if path_from_cwd.exists() {
        return path_from_cwd.to_path_buf();
    }

    // 2. Относительно исполняемого файла
    // Если исполняемый файл в target/debug или target/release, поднимаемся на несколько уровней
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Попробуем подняться из target/debug или target/release к корню проекта
            let path_from_exe = exe_dir
                .parent() // target/
                .and_then(|p| p.parent()) // корень проекта
                .map(|p| p.join("web/dist/index.html"));

            if let Some(ref path) = path_from_exe {
                if path.exists() {
                    return path.clone();
                }
            }

            // Альтернативный вариант: если мы в server/target/..., то нужно подняться выше
            // Проверяем, содержит ли путь "server"
            let mut current = exe_dir;
            while let Some(parent) = current.parent() {
                if parent.join("web/dist/index.html").exists() {
                    return parent.join("web/dist/index.html");
                }
                if parent.join("server").exists() && parent.join("web").exists() {
                    // Это корень проекта
                    return parent.join("web/dist/index.html");
                }
                current = parent;
                // Ограничим глубину поиска
                if current.as_os_str().len() < 10 {
                    break;
                }
            }
        }
    }

    // 3. Относительно текущей директории, если она server/
    let cwd_path = std::env::current_dir()
        .ok()
        .map(|cwd| {
            // Если мы в server/, поднимаемся на уровень выше
            if cwd.ends_with("server") {
                cwd.parent().map(|p| p.join("web/dist/index.html"))
            } else {
                Some(cwd.join("web/dist/index.html"))
            }
        })
        .flatten();

    if let Some(ref path) = cwd_path {
        if path.exists() {
            return path.clone();
        }
    }

    // Fallback: возвращаем путь относительно текущей директории (даже если не существует)
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("web/dist/index.html")
}

/// Handler для SPA fallback - возвращает index.html для всех не-API роутов
async fn serve_index() -> Response<Body> {
    let index_path = get_index_path();

    // Логирование пути для диагностики
    tracing::debug!("Attempting to read index.html from: {:?}", index_path);
    tracing::debug!("File exists: {}", index_path.exists());

    match tokio::fs::read(&index_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(content))
            .unwrap(),
        Err(e) => {
            tracing::error!("Failed to read index.html from {:?}: {}", index_path, e);
            if let Ok(cwd) = std::env::current_dir() {
                tracing::error!("Current working directory: {:?}", cwd);
            }
            if let Ok(exe) = std::env::current_exe() {
                tracing::error!("Executable path: {:?}", exe);
            }
            let error_msg = format!("Failed to read index.html from {:?}: {}", index_path, e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(error_msg))
                .unwrap()
        }
    }
}

/// Создать роутер с всеми маршрутами
pub fn create_router(state: AppState) -> Router {
    use crate::adapters::api::handlers;

    // Статические файлы из web/dist (относительно корня проекта)
    // Используем ту же логику поиска, что и для index.html
    let static_dir = get_index_path().parent().unwrap().to_path_buf();

    // CORS для разработки - разрешаем запросы с localhost:5173 (Vite dev server)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers(Any);

    Router::new()
        // Health check endpoint
        .route("/health", get(|| async { "OK" }))
        // API маршруты (должны быть перед статикой)
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/me", get(handlers::me))
        .route("/api/auth/logout", post(handlers::logout))
        // ZID SSO (Web login)
        .route("/auth/zid/login", get(handlers::zid_login))
        .route("/auth/zid/callback", get(handlers::zid_callback))
        // Проекты (требуют аутентификации)
        .route("/api/projects", post(handlers::create_project))
        .route("/api/projects", get(handlers::list_projects))
        .route("/api/projects/:id", get(handlers::get_project))
        .route("/api/projects/:id", patch(handlers::update_project))
        .route("/api/projects/:id", delete(handlers::delete_project))
        // Конфигурации (требуют аутентификации)
        .route(
            "/api/projects/:id/configurations/:config_id",
            delete(handlers::delete_configuration),
        )
        .route(
            "/api/projects/:id/configurations",
            post(handlers::create_configuration),
        )
        .route(
            "/api/projects/:id/configurations",
            get(handlers::list_configurations),
        )
        // Параметры (требуют аутентификации)
        // ВАЖНО: более специфичный маршрут с :key должен быть определен первым
        .route(
            "/api/projects/:id/configurations/:config/parameters/:key",
            delete(handlers::delete_parameter),
        )
        .route(
            "/api/projects/:id/configurations/:config/parameters",
            put(handlers::set_parameters),
        )
        .route(
            "/api/projects/:id/configurations/:config/parameters",
            get(handlers::get_parameters),
        )
        // API токены (требуют аутентификации)
        .route("/api/tokens", get(handlers::list_tokens))
        .route("/api/tokens", post(handlers::create_token))
        .route("/api/tokens/:id", delete(handlers::delete_token))
        // Раздача статических файлов из web/dist/assets
        .nest_service("/assets", ServeDir::new(static_dir.join("assets")))
        // Раздача других статических файлов (favicon и т.д.)
        .nest_service("/logo.svg", ServeDir::new(static_dir.clone()))
        // SPA fallback: все остальные запросы возвращают index.html
        .fallback(serve_index)
        .layer(cors)
        .with_state(state)
}
