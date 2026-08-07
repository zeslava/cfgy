use axum::{
    Router,
    http::{HeaderValue, Method, header},
    routing::{delete, get, patch, post, put},
};
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
};

use crate::server::adapters::api::handlers::AppState;
use crate::server::adapters::api::static_assets;

/// Создать роутер с всеми маршрутами
pub fn create_router(state: AppState) -> Router {
    use crate::server::adapters::api::handlers;

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
        .route("/auth/zid/login", get(handlers::oidc_login))
        .route("/auth/zid/callback", get(handlers::oidc_callback))
        .route("/oauth/zid/login", get(handlers::oidc_login))
        .route("/oauth/zid/callback", get(handlers::oidc_callback))
        // Новые OIDC-маршруты (провайдеронезависимые)
        .route("/auth/oidc/login", get(handlers::oidc_login))
        .route("/auth/oidc/callback", get(handlers::oidc_callback))
        .route("/oauth/oidc/login", get(handlers::oidc_login))
        .route("/oauth/oidc/callback", get(handlers::oidc_callback))
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
        // SvelteKit: JS/CSS в _app/ (встроены в бинарь)
        .route("/_app/*path", get(static_assets::serve_app_asset))
        .route("/favicon.svg", get(static_assets::serve_root_asset))
        .route("/logo.svg", get(static_assets::serve_root_asset))
        .route("/robots.txt", get(static_assets::serve_root_asset))
        // SPA fallback: все остальные запросы возвращают index.html
        .fallback(static_assets::serve_index)
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
