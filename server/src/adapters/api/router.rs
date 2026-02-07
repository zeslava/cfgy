use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Method, StatusCode, header},
    response::Response,
    routing::{delete, get, patch, post, put},
};
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    services::ServeDir,
    set_header::SetResponseHeaderLayer,
};

use crate::adapters::api::handlers::AppState;

/// Получить путь к index.html, проверяя несколько возможных расположений
/// Все проверки выполняются во время выполнения, чтобы работать независимо от места запуска.
/// Для FreeBSD/системного сервиса задайте CFGY_WEB_ROOT (каталог с index.html, например /var/lib/cfgy_server/web/build).
pub fn get_index_path() -> std::path::PathBuf {
    // 0. Явный путь из переменной окружения (для системного сервиса, например FreeBSD)
    if let Ok(root) = std::env::var("CFGY_WEB_ROOT") {
        let path = std::path::Path::new(&root).join("index.html");
        if path.exists() {
            return path;
        }
    }

    // 1. В Docker контейнере файлы всегда в /app/web/build
    let docker_path = std::path::Path::new("/app/web/build/index.html");
    if docker_path.exists() {
        return docker_path.to_path_buf();
    }

    // 2. Относительно текущей рабочей директории (если запускаем из корня проекта)
    let path_from_cwd = std::path::Path::new("web/build/index.html");
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
                .map(|p| p.join("web/build/index.html"));

            if let Some(ref path) = path_from_exe {
                if path.exists() {
                    return path.clone();
                }
            }

            // Альтернативный вариант: если мы в server/target/..., то нужно подняться выше
            // Проверяем, содержит ли путь "server"
            let mut current = exe_dir;
            while let Some(parent) = current.parent() {
                if parent.join("web/build/index.html").exists() {
                    return parent.join("web/build/index.html");
                }
                if parent.join("server").exists() && parent.join("web").exists() {
                    // Это корень проекта
                    return parent.join("web/build/index.html");
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
                cwd.parent().map(|p| p.join("web/build/index.html"))
            } else {
                Some(cwd.join("web/build/index.html"))
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
        .join("web/build/index.html")
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

    // Статические файлы из web/build (относительно корня проекта)
    // Используем ту же логику поиска, что и для index.html
    let static_dir = get_index_path().parent().unwrap().to_path_buf();

    // Настройка CORS
    // Если установлен ALLOWED_ORIGINS - используем его, иначе разрешаем все (для домашнего использования)
    let cors = if let Ok(allowed_origins) = std::env::var("ALLOWED_ORIGINS") {
        // Если установлен ALLOWED_ORIGINS - ограничиваем CORS указанными источниками
        let origins: Vec<HeaderValue> = allowed_origins
            .split(',')
            .map(|s| s.trim())
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    HeaderValue::from_str(s).ok()
                }
            })
            .collect();

        if origins.is_empty() {
            eprintln!("⚠️  ВНИМАНИЕ: ALLOWED_ORIGINS пуст, разрешаются все источники");
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_headers(Any)
        } else {
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_headers(Any)
        }
    } else {
        // Если ALLOWED_ORIGINS не установлен - разрешаем все (удобно для домашнего использования)
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
            ])
            .allow_headers(Any)
    };

    let mut router = Router::new()
        // Health check endpoint с проверкой БД
        .route("/health", get(handlers::health_check))
        // API маршруты (должны быть перед статикой)
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/session", get(handlers::session))
        .route("/api/auth/me", get(handlers::me))
        .route("/api/auth/logout", post(handlers::logout))
        // ZID OAuth (Web login). Поддержка и /auth/zid и /oauth/zid (ZID может быть настроен на любой)
        .route("/auth/zid/login", get(handlers::zid_login))
        .route("/auth/zid/callback", get(handlers::zid_callback))
        .route("/oauth/zid/login", get(handlers::zid_login))
        .route("/oauth/zid/callback", get(handlers::zid_callback))
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
        // SvelteKit: JS/CSS в _app/
        .nest_service("/_app", ServeDir::new(static_dir.join("_app")))
        .route("/favicon.svg", get(handlers::serve_favicon))
        .route("/logo.svg", get(handlers::serve_logo))
        .route("/robots.txt", get(handlers::serve_robots_txt))
        // SPA fallback: все остальные запросы возвращают index.html
        .fallback(serve_index)
        .layer(cors)
        // Security Headers
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ))
        .with_state(state);

    // HSTS только если используется HTTPS
    let enable_https = std::env::var("ENABLE_HTTPS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if enable_https {
        router = router.layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ));
    }

    router
}
