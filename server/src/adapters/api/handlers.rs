use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapters::api::auth::extract_user;
use crate::application::{
    auth::AuthService, parameters::ParameterService, projects::ProjectService,
};
use crate::config::{Config, ZidConfig};
use crate::ports::{ExternalIdentityRepository, UserRepository};

/// Контекст приложения (все сервисы и репозитории)
#[derive(Clone)]
pub struct AppState {
    pub project_service: Arc<
        ProjectService<
            crate::adapters::persistence::PostgresUserRepository,
            crate::adapters::persistence::PostgresUserRepository,
            crate::adapters::persistence::PostgresUserRepository,
            crate::adapters::crypto::AesCryptoService,
        >,
    >,
    pub parameter_service: Arc<
        ParameterService<
            crate::adapters::persistence::PostgresUserRepository,
            crate::adapters::persistence::PostgresUserRepository,
            crate::adapters::persistence::PostgresUserRepository,
            crate::adapters::crypto::AesCryptoService,
        >,
    >,
    pub auth_service: Arc<
        AuthService<
            crate::adapters::persistence::PostgresUserRepository,
            crate::adapters::persistence::PostgresUserRepository,
        >,
    >,
    pub user_repo: Arc<dyn UserRepository>,
    pub external_identity_repo: Arc<dyn ExternalIdentityRepository>,
    pub db_pool: PgPool,
    /// Корень каталога со статикой (web/build) для раздачи favicon и т.д.
    pub web_root: PathBuf,
}

const CFGY_SESSION_COOKIE_NAME: &str = "cfgy_session";
const ZID_STATE_COOKIE_NAME: &str = "cfgy_zid_state";
const ZID_NEXT_COOKIE_NAME: &str = "cfgy_zid_next";

/// Раздача фавикона из web_root (для SPA при обращении через Rust-сервер)
pub async fn serve_favicon(State(state): State<AppState>) -> Response {
    let path = state.web_root.join("favicon.svg");
    match tokio::fs::read(&path).await {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/svg+xml")
            .body(Body::from(body))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}

/// Раздача логотипа из web_root (для SPA при обращении через Rust-сервер)
pub async fn serve_logo(State(state): State<AppState>) -> Response {
    let path = state.web_root.join("logo.svg");
    match tokio::fs::read(&path).await {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/svg+xml")
            .body(Body::from(body))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}

/// Раздача robots.txt из web_root (для краулеров)
pub async fn serve_robots_txt(State(state): State<AppState>) -> Response {
    let path = state.web_root.join("robots.txt");
    match tokio::fs::read(&path).await {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from(body))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}

// ========== Типы для запросов и ответов ==========

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct ConfigurationResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct CreateConfigurationRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user_id: Uuid,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>, // JWT сессионный токен для создания постоянных токенов
}

#[derive(Serialize)]
pub struct MeResponse {
    pub user_id: Uuid,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

#[derive(Deserialize)]
pub struct SetParametersRequest {
    #[serde(flatten)]
    pub parameters: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Deserialize)]
pub struct ZidLoginQuery {
    /// Куда вернуть пользователя после успешного логина (SPA route).
    /// Пример: /projects
    pub next: Option<String>,
}

/// Query-параметры callback от ZID (OAuth 2.0)
#[derive(Deserialize)]
pub struct ZidCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Ответ POST /oauth/token от ZID
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ZidTokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub id_token: Option<String>,
}

/// Ответ GET /oauth/userinfo от ZID (OIDC)
#[derive(Deserialize, Debug)]
pub struct ZidUserinfoResponse {
    pub sub: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub preferred_username: Option<String>,
}

/// OIDC Discovery: .well-known/openid-configuration
#[derive(Deserialize, Debug)]
struct OidcDiscovery {
    #[serde(rename = "authorization_endpoint")]
    authorization_endpoint: String,
    #[serde(rename = "token_endpoint")]
    token_endpoint: String,
    #[serde(rename = "userinfo_endpoint")]
    userinfo_endpoint: Option<String>,
}

/// Извлечь из PEM-данных все блоки сертификатов (один или несколько) и добавить их в builder.
fn add_ca_certs_from_pem(
    builder: reqwest::ClientBuilder,
    pem_bytes: &[u8],
) -> Result<reqwest::ClientBuilder, String> {
    const BEGIN: &[u8] = b"-----BEGIN CERTIFICATE-----";
    const END: &[u8] = b"-----END CERTIFICATE-----";
    let mut builder = builder;
    let mut pos = 0;
    while let Some(start) = pem_bytes[pos..]
        .windows(BEGIN.len())
        .position(|w| w == BEGIN)
    {
        let start = pos + start;
        let after_begin = start + BEGIN.len();
        let end_pos = pem_bytes[after_begin..]
            .windows(END.len())
            .position(|w| w == END)
            .map(|i| after_begin + i + END.len())
            .ok_or_else(|| "PEM: не найден -----END CERTIFICATE-----".to_string())?;
        let block = &pem_bytes[start..end_pos];
        let cert =
            reqwest::Certificate::from_pem(block).map_err(|e| format!("PEM сертификат: {}", e))?;
        builder = builder.add_root_certificate(cert);
        pos = end_pos;
    }
    Ok(builder)
}

/// Собрать HTTP-клиент для запросов к ZID (опционально: доверенный CA из файла и/или отключение проверки TLS).
fn build_zid_http_client(zid: &ZidConfig) -> reqwest::Client {
    let mut builder = reqwest::Client::builder();
    if let Some(ref path) = zid.ca_file {
        if let Ok(pem) = std::fs::read(path) {
            if let Ok(b) = add_ca_certs_from_pem(reqwest::Client::builder(), &pem) {
                builder = b;
            }
        }
    }
    if zid.tls_insecure {
        builder = builder.danger_accept_invalid_certs(true);
    }
    builder.build().expect("reqwest Client для ZID")
}

/// Собрать URL для запроса с бэкенда: подставить base вместо хоста из discovery_url, путь взять из discovery_url.
fn override_base_url(base: &str, discovery_url: &str) -> String {
    let path = if let Some((_, rest)) = discovery_url.split_once("://") {
        let from_slash = rest.find('/').map(|i| &rest[i..]).unwrap_or("/");
        let path_only = from_slash
            .split_once('?')
            .map(|(p, _)| p)
            .unwrap_or(from_slash);
        path_only
    } else {
        discovery_url
    };
    format!("{}{}", base.trim_end_matches('/'), path)
}

/// Получить endpoints ZID: из discovery или захардкоженные пути.
/// discovery_base — по нему запрашивается /.well-known/openid-configuration.
/// backend_requests_base — если задан, token и userinfo собираются как base + path из discovery (для запросов с бэкенда по ZID_BASE_URL).
async fn get_zid_endpoints(
    discovery_base: &str,
    backend_requests_base: Option<&str>,
    client: &reqwest::Client,
) -> (String, String, String) {
    let base = discovery_base.trim_end_matches('/');
    let base = if base.is_empty() || (!base.starts_with("http://") && !base.starts_with("https://"))
    {
        "http://localhost:5555"
    } else {
        base
    };
    let url = format!("{}/.well-known/openid-configuration", base);
    if let Ok(resp) = client.get(&url).send().await
        && resp.status().is_success()
        && let Ok(discovery) = resp.json::<OidcDiscovery>().await
    {
        let (token_endpoint, userinfo_endpoint) = match backend_requests_base {
            Some(b) => (
                override_base_url(b, &discovery.token_endpoint),
                override_base_url(
                    b,
                    discovery
                        .userinfo_endpoint
                        .as_deref()
                        .unwrap_or("/oauth/userinfo"),
                ),
            ),
            None => (
                discovery.token_endpoint,
                discovery
                    .userinfo_endpoint
                    .unwrap_or_else(|| format!("{}/oauth/userinfo", base)),
            ),
        };
        return (
            discovery.authorization_endpoint,
            token_endpoint,
            userinfo_endpoint,
        );
    }
    (
        format!("{}/oauth/authorize", base),
        format!("{}/oauth/token", base),
        format!("{}/oauth/userinfo", base),
    )
}

/// Параметры пути для DELETE параметра
/// Извлекается из URL: /api/projects/:id/configurations/:config/parameters/:key
/// Имена полей структуры должны совпадать с именами параметров в URL
#[derive(Deserialize)]
pub struct DeleteParameterPath {
    pub id: String,
    pub config: String,
    pub key: String,
}

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    pub project_id: Option<uuid::Uuid>,
    pub expires_in_days: Option<u32>,
}

#[derive(Serialize)]
pub struct CreateTokenResponse {
    pub token: String,
    pub token_id: uuid::Uuid,
    pub name: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub project_id: Option<uuid::Uuid>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ========== Handlers ==========

/// POST /api/auth/register
///
/// Возвращает session_token в JSON. Фронт сохраняет и шлёт Authorization: Bearer.
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), (StatusCode, Json<ErrorResponse>)> {
    let user = state
        .auth_service
        .register(&request.email, &request.password)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    let session_token = state.auth_service.create_session_token(user.id).ok();
    Ok((
        StatusCode::CREATED,
        Json(LoginResponse {
            user_id: user.id,
            email: user.email,
            session_token,
        }),
    ))
}

/// GET /auth/zid/login
///
/// OAuth 2.0 Authorization Code: генерирует state, сохраняет в cookie вместе с next,
/// формирует URL авторизации ZID и редиректит пользователя на страницу входа ZID.
pub async fn zid_login(Query(q): Query<ZidLoginQuery>) -> impl IntoResponse {
    let config = Config::load().unwrap_or_default();

    if config.zid.client_id.is_empty() || config.zid.client_secret.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "ZID OAuth не настроен (ZID_CLIENT_ID / ZID_CLIENT_SECRET)".to_string(),
            }),
        )
            .into_response();
    }

    let state = Uuid::new_v4().to_string();
    let next = q.next.unwrap_or_else(|| "/projects".to_string());

    // Редирект в браузере — всегда по публичному issuer (доступен с клиента пользователя)
    let issuer_base = config.zid.issuer.trim_end_matches('/');
    let issuer_base = if issuer_base.is_empty()
        || (!issuer_base.starts_with("http://") && !issuer_base.starts_with("https://"))
    {
        "http://localhost:5555"
    } else {
        issuer_base
    };
    let authorization_endpoint = format!("{}/oauth/authorize", issuer_base);
    let redirect_uri_encoded = urlencoding::encode(&config.zid.redirect_uri);
    let zid_login_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope=openid%20profile%20email&state={}",
        authorization_endpoint,
        urlencoding::encode(&config.zid.client_id),
        redirect_uri_encoded,
        urlencoding::encode(&state)
    );

    let secure = config.zid.redirect_uri.starts_with("https://");
    let secure_flag = if secure { "; Secure" } else { "" };
    let set_state_cookie = format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age=300{secure_flag}",
        name = ZID_STATE_COOKIE_NAME,
        value = state,
        secure_flag = secure_flag
    );
    let set_next_cookie = format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age=300{secure_flag}",
        name = ZID_NEXT_COOKIE_NAME,
        value = urlencoding::encode(&next),
        secure_flag = secure_flag
    );

    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&set_state_cookie).unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&set_next_cookie).unwrap(),
    );
    (headers, Redirect::temporary(&zid_login_url)).into_response()
}

/// Вспомогательная: прочитать значение cookie из заголовка Cookie
fn get_cookie_value(cookie_header: &str, name: &str) -> Option<String> {
    for part in cookie_header.split(';') {
        let p = part.trim();
        if let Some(rest) = p.strip_prefix(&format!("{}=", name)) {
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

/// Вернуть редирект на /login с ошибкой и очисткой state/next cookies
fn zid_callback_error_redirect(
    message: &str,
    secure: bool,
) -> (HeaderMap, axum::response::Redirect) {
    let secure_flag = if secure { "; Secure" } else { "" };
    let clear_state = format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure_flag}",
        name = ZID_STATE_COOKIE_NAME,
        secure_flag = secure_flag
    );
    let clear_next = format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure_flag}",
        name = ZID_NEXT_COOKIE_NAME,
        secure_flag = secure_flag
    );
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_state).unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_next).unwrap(),
    );
    let login_url = format!("/login?error={}", urlencoding::encode(message));
    (headers, Redirect::temporary(&login_url))
}

/// GET /auth/zid/callback
///
/// OAuth 2.0 callback от ZID: проверяет state, обменивает code на токены на бэкенде,
/// при необходимости запрашивает userinfo, находит/создаёт пользователя в CFGY,
/// создаёт сессию (JWT в cookie) и редиректит в next.
pub async fn zid_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ZidCallbackQuery>,
) -> impl IntoResponse {
    let config = Config::load().unwrap_or_default();
    let secure = config.zid.redirect_uri.starts_with("https://");

    if let Some(ref err) = q.error {
        let mut msg = q.error_description.as_deref().unwrap_or(err).to_string();
        // Подсказка при типичной ошибке: redirect_uri не совпадает с настройками в ZID
        if err == "invalid_request"
            && q.error_description
                .as_deref()
                .map_or(true, |d| !d.contains("redirect_uri"))
        {
            msg.push_str(" (проверьте: ZID_REDIRECT_URI на сервере CFGY и список redirect_uri в настройках клиента ZID должны совпадать побайтово)");
        }
        return zid_callback_error_redirect(&msg, secure).into_response();
    }

    let code = match &q.code {
        Some(c) if !c.is_empty() => c.clone(),
        _ => {
            return zid_callback_error_redirect("Отсутствует параметр code", secure)
                .into_response();
        }
    };

    let state_from_query = match &q.state {
        Some(s) if !s.is_empty() => s.clone(),
        _ => {
            return zid_callback_error_redirect("Отсутствует параметр state", secure)
                .into_response();
        }
    };

    let cookie_header = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let state_from_cookie = match get_cookie_value(cookie_header, ZID_STATE_COOKIE_NAME) {
        Some(s) => s,
        None => {
            return zid_callback_error_redirect("Неверный или истёкший state", secure)
                .into_response();
        }
    };
    if state_from_cookie != state_from_query {
        return zid_callback_error_redirect("Неверный state (CSRF)", secure).into_response();
    }

    let next_encoded = get_cookie_value(cookie_header, ZID_NEXT_COOKIE_NAME);
    let next = next_encoded
        .as_ref()
        .and_then(|s| urlencoding::decode(s).ok())
        .map(|c| c.into_owned())
        .unwrap_or_else(|| "/projects".to_string());

    // Запросы бэкенда к ZID (token, userinfo) — по base_url, если задан; иначе по issuer
    let backend_base = config
        .zid
        .base_url
        .as_deref()
        .unwrap_or(config.zid.issuer.as_str());
    let client = build_zid_http_client(&config.zid);
    let (_, token_endpoint, userinfo_endpoint) =
        get_zid_endpoints(backend_base, config.zid.base_url.as_deref(), &client).await;

    let token_body = format!(
        "grant_type=authorization_code&client_id={}&client_secret={}&redirect_uri={}&code={}",
        urlencoding::encode(&config.zid.client_id),
        urlencoding::encode(&config.zid.client_secret),
        urlencoding::encode(&config.zid.redirect_uri),
        urlencoding::encode(&code)
    );

    let token_resp = match client
        .post(&token_endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(token_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            let mut msg = format!("Ошибка запроса токена к ZID: {}", e);
            if let Some(source) = e.source() {
                msg.push_str(&format!(" (причина: {})", source));
            }
            return zid_callback_error_redirect(&msg, secure).into_response();
        }
    };

    if !token_resp.status().is_success() {
        let text = token_resp.text().await.unwrap_or_default();
        return zid_callback_error_redirect(&format!("ZID отклонил обмен кода: {}", text), secure)
            .into_response();
    }

    let token_data: ZidTokenResponse = match token_resp.json().await {
        Ok(t) => t,
        Err(e) => {
            return zid_callback_error_redirect(
                &format!("Не удалось разобрать ответ ZID: {}", e),
                secure,
            )
            .into_response();
        }
    };

    tracing::debug!(
        "ZID token response: token_type={:?}, has_id_token={}, expires_in={:?}",
        token_data.token_type,
        token_data.id_token.is_some(),
        token_data.expires_in
    );

    let access_token = token_data.access_token.clone();
    let id_token = token_data.id_token.clone();
    let mut sub = String::new();
    let mut username = String::new();
    // Реальный email из ZID (userinfo) — используем при создании пользователя, чтобы не подставлять @zid.local при наличии email.
    let mut email_from_zid: Option<String> = None;

    // Запрос userinfo: часть провайдеров принимает access_token, часть — id_token (JWT).
    let (userinfo_status, userinfo_raw) = {
        let resp = client
            .get(&userinfo_endpoint)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await;
        let (status, body) = match resp {
            Ok(r) => (r.status(), r.text().await.unwrap_or_default()),
            Err(e) => {
                tracing::warn!("ZID userinfo запрос не удался: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, String::new())
            }
        };
        tracing::debug!(
            "ZID userinfo (Bearer access_token): url={}, status={}, body={}",
            userinfo_endpoint,
            status,
            body
        );
        if status.is_success() {
            (status, body)
        } else if status.as_u16() == 401 && id_token.as_deref().map_or(false, |t| !t.is_empty()) {
            // 401 — пробуем id_token в заголовке
            tracing::debug!("ZID userinfo 401, повтор с Bearer id_token");
            let resp2 = client
                .get(&userinfo_endpoint)
                .header(
                    "Authorization",
                    format!("Bearer {}", id_token.as_deref().unwrap_or_default()),
                )
                .send()
                .await;
            let (status2, body2) = match resp2 {
                Ok(r) => (r.status(), r.text().await.unwrap_or_default()),
                Err(e) => {
                    tracing::warn!("ZID userinfo (id_token) запрос не удался: {}", e);
                    (status, body)
                }
            };
            tracing::debug!(
                "ZID userinfo (Bearer id_token): status={}, body={}",
                status2,
                body2
            );
            if status2.is_success() {
                (status2, body2)
            } else if status2.as_u16() == 401 {
                // Оба Bearer не подошли — пробуем access_token в query (?access_token=...)
                let url_with_token = format!(
                    "{}?access_token={}",
                    userinfo_endpoint,
                    urlencoding::encode(&access_token)
                );
                tracing::debug!("ZID userinfo 401, повтор с query access_token");
                let resp3 = client.get(&url_with_token).send().await;
                let (status3, body3) = match resp3 {
                    Ok(r) => (r.status(), r.text().await.unwrap_or_default()),
                    Err(e) => {
                        tracing::warn!("ZID userinfo (?access_token) запрос не удался: {}", e);
                        (status2, body2)
                    }
                };
                tracing::debug!(
                    "ZID userinfo (?access_token): status={}, body={}",
                    status3,
                    body3
                );
                (status3, body3)
            } else {
                (status2, body2)
            }
        } else {
            (status, body)
        }
    };

    if userinfo_status.is_success() {
        if let Ok(userinfo) = serde_json::from_str::<ZidUserinfoResponse>(&userinfo_raw) {
            tracing::debug!(
                "ZID userinfo parsed: sub={}, name={:?}, email={:?}, preferred_username={:?}",
                userinfo.sub,
                userinfo.name,
                userinfo.email,
                userinfo.preferred_username
            );
            sub = userinfo.sub.clone();
            email_from_zid = userinfo.email.clone();
            username = userinfo
                .preferred_username
                .or(userinfo.name)
                .or(email_from_zid.clone())
                .unwrap_or_else(|| sub.clone());
        } else {
            tracing::warn!(
                "ZID userinfo не удалось распарсить как ZidUserinfoResponse, сырой body: {}",
                userinfo_raw
            );
        }
    }

    // Если userinfo не отдал данные (401 и т.д.) — берём из id_token (JWT payload).
    if sub.is_empty() || (username.is_empty() && email_from_zid.is_none()) {
        if let Some(ref id_token) = token_data.id_token {
            if let Some(claims) = parse_claims_from_id_token(id_token) {
                tracing::debug!(
                    "ZID id_token claims: sub={}, name={:?}, email={:?}, preferred_username={:?}",
                    claims.sub,
                    claims.name,
                    claims.email,
                    claims.preferred_username
                );
                if sub.is_empty() {
                    sub = claims.sub;
                }
                if email_from_zid.is_none() {
                    email_from_zid = claims.email.clone();
                }
                if username.is_empty() {
                    username = claims
                        .preferred_username
                        .or(claims.name)
                        .or(claims.email)
                        .unwrap_or_else(|| sub.clone());
                }
            }
        }
    }

    if sub.is_empty() {
        return zid_callback_error_redirect(
            "Не удалось получить идентификатор пользователя из ZID",
            secure,
        )
        .into_response();
    }
    if username.is_empty() {
        username = sub.clone();
    }

    // Вычисляем email один раз — используем при создании/обновлении и при обновлении профиля ниже.
    let email_to_save = email_from_zid.unwrap_or_else(|| {
        if username.contains('@') {
            username.clone()
        } else {
            format!("{}@zid.local", username)
        }
    });

    let maybe_user_id = match state.external_identity_repo.find_user_id("zid", &sub).await {
        Ok(v) => v,
        Err(e) => {
            return zid_callback_error_redirect(&format!("Ошибка поиска привязки: {}", e), secure)
                .into_response();
        }
    };

    let user_id = if let Some(uid) = maybe_user_id {
        uid
    } else {
        let existing = match state.user_repo.find_by_username(&username).await {
            Ok(v) => v,
            Err(e) => {
                return zid_callback_error_redirect(
                    &format!("Ошибка поиска пользователя: {}", e),
                    secure,
                )
                .into_response();
            }
        };
        let user = if let Some(u) = existing {
            u
        } else {
            match state
                .user_repo
                .create_sso_user(&email_to_save, &username)
                .await
            {
                Ok(u) => u,
                Err(e) => {
                    return zid_callback_error_redirect(
                        &format!("Ошибка создания пользователя: {}", e),
                        secure,
                    )
                    .into_response();
                }
            }
        };
        if let Err(e) = state
            .external_identity_repo
            .link_user(user.id, "zid", &sub)
            .await
        {
            return zid_callback_error_redirect(
                &format!("Ошибка привязки идентичности: {}", e),
                secure,
            )
            .into_response();
        }
        user.id
    };

    // Обновляем email/username из ZID при каждом входе (актуальные данные из провайдера).
    if let Err(e) = state
        .user_repo
        .update_profile(user_id, &email_to_save, &username)
        .await
    {
        return zid_callback_error_redirect(&format!("Ошибка обновления профиля: {}", e), secure)
            .into_response();
    }

    let session_token = match state.auth_service.create_session_token(user_id) {
        Ok(t) => t,
        Err(e) => {
            return zid_callback_error_redirect(&format!("Ошибка создания сессии: {}", e), secure)
                .into_response();
        }
    };

    let secure_flag = if secure { "; Secure" } else { "" };
    // В dev (localhost) ставим Domain=localhost, чтобы cookie работала и на 5173, и на 8080
    let domain_flag = if config.zid.redirect_uri.contains("localhost") {
        "; Domain=localhost"
    } else {
        ""
    };
    let set_session_cookie = format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400{domain_flag}{secure_flag}",
        name = CFGY_SESSION_COOKIE_NAME,
        value = session_token,
        domain_flag = domain_flag,
        secure_flag = secure_flag
    );
    let clear_state_cookie = format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{domain_flag}{secure_flag}",
        name = ZID_STATE_COOKIE_NAME,
        domain_flag = domain_flag,
        secure_flag = secure_flag
    );
    let clear_next_cookie = format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{domain_flag}{secure_flag}",
        name = ZID_NEXT_COOKIE_NAME,
        domain_flag = domain_flag,
        secure_flag = secure_flag
    );

    let mut out_headers = HeaderMap::new();
    out_headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&set_session_cookie).unwrap(),
    );
    out_headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_state_cookie).unwrap(),
    );
    out_headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_next_cookie).unwrap(),
    );

    // Абсолютный URL редиректа: браузер после прокси должен уйти на тот же origin (5173), а не на 8080
    let base = {
        let uri = &config.zid.redirect_uri;
        if let Some(prefix) = uri.find("://") {
            let after = &uri[prefix + 3..];
            if let Some(slash) = after.find('/') {
                uri[..prefix + 3 + slash].to_string()
            } else {
                uri.to_string()
            }
        } else {
            String::new()
        }
    };
    let redirect_url = if base.is_empty() {
        format!(
            "{}#session_token={}",
            next,
            urlencoding::encode(&session_token)
        )
    } else {
        let path = next.strip_prefix('/').unwrap_or(next.as_str());
        format!(
            "{}/{}#session_token={}",
            base,
            path,
            urlencoding::encode(&session_token)
        )
    };
    (out_headers, Redirect::temporary(&redirect_url)).into_response()
}

/// Claims из payload id_token (JWT) — sub и опционально email, name, preferred_username.
/// Без верификации подписи (достаточно для заполнения профиля при 401 от userinfo).
fn parse_claims_from_id_token(id_token: &str) -> Option<IdTokenClaims> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload_b64 = parts[1];
    use base64::Engine;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .ok()?;
    let payload: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    let sub = payload
        .get("sub")
        .and_then(|v| v.as_str())
        .map(String::from)?;
    let email = payload
        .get("email")
        .and_then(|v| v.as_str())
        .map(String::from);
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .map(String::from);
    let preferred_username = payload
        .get("preferred_username")
        .and_then(|v| v.as_str())
        .map(String::from);
    Some(IdTokenClaims {
        sub,
        email,
        name,
        preferred_username,
    })
}

struct IdTokenClaims {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    preferred_username: Option<String>,
}

/// GET /api/auth/session
///
/// Обменивает cookie сессии (после OAuth redirect) на токен в теле ответа.
/// Фронт вызывает с credentials: include, получает session_token, сохраняет и дальше шлёт Authorization: Bearer.
/// Cookie после ответа очищается (одноразовый обмен).
pub async fn session(headers: HeaderMap) -> impl IntoResponse {
    let config = Config::load().unwrap_or_default();
    let secure = config.zid.redirect_uri.starts_with("https://");
    let domain_flag = if config.zid.redirect_uri.contains("localhost") {
        "; Domain=localhost"
    } else {
        ""
    };
    let secure_flag = if secure { "; Secure" } else { "" };
    let clear_cookie = format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{domain_flag}{secure_flag}",
        name = CFGY_SESSION_COOKIE_NAME,
        domain_flag = domain_flag,
        secure_flag = secure_flag
    );

    let cookie_header = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = get_cookie_value(cookie_header, CFGY_SESSION_COOKIE_NAME);

    let mut res = match token {
        Some(t) => (
            StatusCode::OK,
            Json(serde_json::json!({ "session_token": t })),
        )
            .into_response(),
        None => (StatusCode::NO_CONTENT, ()).into_response(),
    };
    res.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_cookie).unwrap(),
    );
    res
}

/// GET /api/auth/me
///
/// Возвращает текущего пользователя.
/// Поддерживает Authorization: Bearer (session token) и cookie (для одноразового обмена через /api/auth/session).
pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MeResponse>, (StatusCode, Json<ErrorResponse>)> {
    let authed = extract_user(&headers, &state).await?;

    let user = state
        .user_repo
        .find_by_id(authed.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("Failed to load user: {}", e),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                }),
            )
        })?;

    Ok(Json(MeResponse {
        user_id: user.id,
        email: user.email,
        username: user.username,
    }))
}

/// GET /health
///
/// Health check endpoint с проверкой доступности базы данных
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Проверка доступности БД через простой запрос
    match sqlx::query("SELECT 1").execute(&state.db_pool).await {
        Ok(_) => (StatusCode::OK, "OK"),
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, "DB unavailable")
        }
    }
}

/// POST /api/auth/logout
///
/// Очищает httpOnly cookie веб-сессии (`cfgy_session`).
pub async fn logout() -> impl IntoResponse {
    let config = Config::load().unwrap_or_default();
    let secure = config.zid.redirect_uri.starts_with("https://");
    let domain_flag = if config.zid.redirect_uri.contains("localhost") {
        "; Domain=localhost"
    } else {
        ""
    };
    let secure_flag = if secure { "; Secure" } else { "" };

    let clear_session_cookie = format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{domain_flag}{secure_flag}",
        name = CFGY_SESSION_COOKIE_NAME,
        domain_flag = domain_flag,
        secure_flag = secure_flag
    );

    let mut out_headers = HeaderMap::new();
    out_headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_session_cookie).unwrap(),
    );

    (out_headers, StatusCode::NO_CONTENT)
}

/// POST /api/auth/login
///
/// Возвращает session_token в JSON. Фронт сохраняет и шлёт Authorization: Bearer.
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = state
        .auth_service
        .authenticate(&request.email, &request.password)
        .await
        .map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    let session_token = state.auth_service.create_session_token(user.id).ok();
    Ok(Json(LoginResponse {
        user_id: user.id,
        email: user.email,
        session_token,
    }))
}

/// POST /api/projects
pub async fn create_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    let project = state
        .project_service
        .create_project(&request.name, request.description.as_deref(), user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(ProjectResponse {
            id: project.id,
            name: project.name,
            description: project.description,
            owner_id: project.owner_id,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }),
    ))
}

/// GET /api/projects
pub async fn list_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProjectResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    let projects = state
        .project_service
        .list_projects(user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(
        projects
            .into_iter()
            .map(|p| ProjectResponse {
                id: p.id,
                name: p.name,
                description: p.description,
                owner_id: p.owner_id,
                created_at: p.created_at,
                updated_at: p.updated_at,
            })
            .collect(),
    ))
}

/// GET /api/projects/:id
pub async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    let project = state.project_service.get_project(id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    if project.owner_id != user.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "У вас нет доступа к этому проекту".to_string(),
            }),
        ));
    }

    Ok(Json(ProjectResponse {
        id: project.id,
        name: project.name,
        description: project.description,
        owner_id: project.owner_id,
        created_at: project.created_at,
        updated_at: project.updated_at,
    }))
}

/// PATCH /api/projects/:id
pub async fn update_project(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    let project = state
        .project_service
        .update_project(
            id,
            user.user_id,
            request.name.as_deref(),
            request.description.as_deref(),
        )
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(ProjectResponse {
        id: project.id,
        name: project.name,
        description: project.description,
        owner_id: project.owner_id,
        created_at: project.created_at,
        updated_at: project.updated_at,
    }))
}

/// DELETE /api/projects/:id
pub async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    state
        .project_service
        .delete_project(id, user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/projects/:id/configurations
pub async fn create_configuration(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<CreateConfigurationRequest>,
) -> Result<(StatusCode, Json<ConfigurationResponse>), (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    let config = state
        .project_service
        .create_configuration(project_id, user.user_id, &request.name)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(ConfigurationResponse {
            id: config.id,
            project_id: config.project_id,
            name: config.name,
            created_at: config.created_at,
            updated_at: config.updated_at,
        }),
    ))
}

/// GET /api/projects/:id/configurations
pub async fn list_configurations(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Vec<ConfigurationResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    // Проверяем, что пользователь имеет доступ к проекту
    let project = state
        .project_service
        .get_project(project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    if project.owner_id != user.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "У вас нет доступа к этому проекту".to_string(),
            }),
        ));
    }

    let configs = state
        .project_service
        .list_configurations(project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(
        configs
            .into_iter()
            .map(|c| ConfigurationResponse {
                id: c.id,
                project_id: c.project_id,
                name: c.name,
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect(),
    ))
}

/// DELETE /api/projects/:id/configurations/:config_id
pub async fn delete_configuration(
    State(state): State<AppState>,
    Path((_project_id, config_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;

    state
        .project_service
        .delete_configuration(config_id, user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /api/projects/:id/configurations/:config/parameters
pub async fn set_parameters(
    State(state): State<AppState>,
    Path((project_id, config_name)): Path<(Uuid, String)>,
    headers: HeaderMap,
    Json(request): Json<SetParametersRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    state
        .parameter_service
        .set_parameters(project_id, &config_name, user.user_id, &request.parameters)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/projects/:id/configurations/:config/parameters
pub async fn get_parameters(
    State(state): State<AppState>,
    Path((project_id, config_name)): Path<(Uuid, String)>,
    headers: HeaderMap,
) -> Result<Json<HashMap<String, String>>, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    let parameters = state
        .parameter_service
        .list_parameters(project_id, &config_name, user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(parameters))
}

/// DELETE /api/projects/:id/configurations/:config/parameters/:key
pub async fn delete_parameter(
    State(state): State<AppState>,
    Path(params): Path<DeleteParameterPath>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Преобразуем project_id из String в Uuid
    let project_id = params.id.parse::<Uuid>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Неверный формат UUID проекта: {}", params.id),
            }),
        )
    })?;
    let user = extract_user(&headers, &state).await?;
    let deleted = state
        .parameter_service
        .delete_parameter(project_id, &params.config, user.user_id, &params.key)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!(
                    "Параметр '{}' не найден в конфигурации '{}'",
                    params.key, params.config
                ),
            }),
        ))
    }
}

/// GET /api/tokens
pub async fn list_tokens(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<TokenResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;

    let tokens = state
        .auth_service
        .list_access_tokens(user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    let token_responses: Vec<TokenResponse> = tokens
        .into_iter()
        .map(|t| TokenResponse {
            id: t.id,
            name: t.name,
            project_id: t.project_id,
            expires_at: t.expires_at,
            created_at: t.created_at,
            last_used_at: t.last_used_at,
        })
        .collect();

    Ok(Json(token_responses))
}

/// POST /api/tokens
pub async fn create_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateTokenRequest>,
) -> Result<(StatusCode, Json<CreateTokenResponse>), (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;

    let (access_token, token) = state
        .auth_service
        .create_access_token(
            user.user_id,
            request.project_id,
            &request.name,
            request.expires_in_days,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(CreateTokenResponse {
            token,
            token_id: access_token.id,
            name: access_token.name,
            expires_at: access_token.expires_at,
        }),
    ))
}

/// DELETE /api/tokens/:id
pub async fn delete_token(
    State(state): State<AppState>,
    Path(token_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;

    state
        .auth_service
        .delete_access_token(token_id, user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}
