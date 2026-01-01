use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Json, Redirect},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapters::api::auth::extract_user;
use crate::application::{
    auth::AuthService, parameters::ParameterService, projects::ProjectService,
};
use crate::config::Config;
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
}

const CFGY_SESSION_COOKIE_NAME: &str = "cfgy_session";
const ZID_STATE_COOKIE_NAME: &str = "cfgy_zid_state";

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

#[derive(Deserialize)]
pub struct ZidCallbackQuery {
    pub ticket: Option<String>,
    pub state: Option<String>,
    /// Куда вернуть пользователя после успешного логина (SPA route).
    /// Пример: /projects
    pub next: Option<String>,
    /// Поддержка вариантов, когда ZID кладёт ticket в другой параметр
    pub code: Option<String>,
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ZidVerifyRequest {
    pub ticket: String,
    /// service должен совпадать 1:1 с return_to/callback URL, который был передан в ZID
    pub service: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ZidVerifyResponse {
    pub success: bool,
    pub user_id: String,
    pub username: String,
    pub session_id: String,
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

    // Создать JWT сессионный токен
    let session_token = state.auth_service.create_session_token(user.id).ok(); // Если не удалось создать, продолжаем без токена

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
/// Генерирует state и редиректит в ZID.
/// Важно: state сохраняем в httpOnly cookie.
/// `callback_url` берём из конфигурации сервера, но в него добавляем query state и next.
///
/// Пример callback_url (dev через web): http://localhost:5173/auth/zid/callback
pub async fn zid_login(Query(q): Query<ZidLoginQuery>) -> impl IntoResponse {
    let config = Config::load().unwrap_or_default();
    eprintln!(
        "[ZID] login: next={:?}, callback_url={}",
        q.next, config.zid.callback_url
    );

    // state для защиты от CSRF/replay
    let state = Uuid::new_v4().to_string();

    // куда вернёмся в SPA после успешной сессии
    let next = q.next.unwrap_or_else(|| "/projects".to_string());

    // callback должен идти через web, но этот endpoint исполняется на server.
    // Мы добавляем параметры в callback_url через query.
    let callback_with_params = format!(
        "{}?state={}&next={}",
        config.zid.callback_url,
        urlencoding::encode(&state),
        urlencoding::encode(&next)
    );

    // ZID ожидает return_to; по вашей договорённости используем корневой `/` у zid.
    let zid_login_url = format!(
        "{}/?return_to={}",
        config.zid.base_url.trim_end_matches('/'),
        urlencoding::encode(&callback_with_params)
    );

    // state cookie: httpOnly, SameSite=Lax (чтобы работало при redirect), Path=/
    // Secure в проде должен быть true (здесь ставим условно по схеме callback_url).
    let secure = config.zid.callback_url.starts_with("https://");
    let set_cookie = format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age=300{secure_flag}",
        name = ZID_STATE_COOKIE_NAME,
        value = state,
        secure_flag = if secure { "; Secure" } else { "" }
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&set_cookie).unwrap(),
    );
    (headers, Redirect::temporary(&zid_login_url))
}

/// GET /auth/zid/callback
///
/// Обрабатывает callback от ZID:
/// - проверяет state (query vs cookie)
/// - вызывает ZID /verify
/// - находит/создаёт пользователя в CFGY и связывает его с ZID user_id
/// - создаёт локальную cfgy-сессию (JWT) и кладёт в httpOnly cookie
/// - редиректит в `next` (SPA route)
pub async fn zid_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ZidCallbackQuery>,
    axum::extract::RawQuery(raw_query): axum::extract::RawQuery,
) -> impl IntoResponse {
    let config = Config::load().unwrap_or_default();

    // Логируем для отладки
    eprintln!(
        "[ZID] callback: raw_query={:?}, ticket={:?}, code={:?}, token={:?}, state={:?}, next={:?}",
        raw_query, q.ticket, q.code, q.token, q.state, q.next
    );

    // ZID может добавить ticket в next параметр вместо отдельного query param
    // Пример: next=/projects?ticket=xxx
    let (ticket, next_clean) = if let Some(next_val) = &q.next {
        if let Some(pos) = next_val.find("?ticket=") {
            let ticket_part = &next_val[pos + 8..];
            let next_clean = next_val[..pos].to_string();
            (Some(ticket_part.to_string()), Some(next_clean))
        } else if let Some(pos) = next_val.find("&ticket=") {
            let ticket_part = &next_val[pos + 8..];
            let next_clean = next_val[..pos].to_string();
            (Some(ticket_part.to_string()), Some(next_clean))
        } else {
            (None, q.next.clone())
        }
    } else {
        (None, q.next.clone())
    };

    let ticket = q
        .ticket
        .or(q.code)
        .or(q.token)
        .or(ticket)
        .unwrap_or_else(|| "".to_string());
    let state_from_query = q.state.unwrap_or_else(|| "".to_string());

    eprintln!(
        "[ZID] parsed: ticket={}, next_clean={:?}",
        ticket, next_clean
    );

    if ticket.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing ticket".to_string(),
            }),
        )
            .into_response();
    }

    if state_from_query.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing state".to_string(),
            }),
        )
            .into_response();
    }

    // достаём state из cookie, сравниваем с query
    let cookie_header = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let mut state_from_cookie: Option<String> = None;
    for part in cookie_header.split(';') {
        let p = part.trim();
        if let Some(rest) = p.strip_prefix(&format!("{}=", ZID_STATE_COOKIE_NAME)) {
            state_from_cookie = Some(rest.to_string());
            break;
        }
    }

    if state_from_cookie.as_deref() != Some(state_from_query.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid state".to_string(),
            }),
        )
            .into_response();
    }

    // service должен совпадать 1:1 с return_to
    // В login мы добавляли query-параметры state/next внутрь callback_url,
    // поэтому здесь собираем такой же service.
    let next = next_clean.unwrap_or_else(|| "/projects".to_string());
    let service = format!(
        "{}?state={}&next={}",
        config.zid.callback_url,
        urlencoding::encode(&state_from_query),
        urlencoding::encode(&next)
    );

    let verify_url = format!("{}/verify", config.zid.base_url.trim_end_matches('/'));
    let body = ZidVerifyRequest { ticket, service };

    let client = reqwest::Client::new();
    let verify_resp = match client.post(verify_url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("ZID verify request failed: {}", e),
                }),
            )
                .into_response();
        }
    };

    if !verify_resp.status().is_success() {
        let text = verify_resp.text().await.unwrap_or_else(|_| "".to_string());
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: format!("ZID verify failed: {}", text),
            }),
        )
            .into_response();
    }

    let zid_identity: ZidVerifyResponse = match verify_resp.json().await {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("ZID verify response parse failed: {}", e),
                }),
            )
                .into_response();
        }
    };

    if !zid_identity.success {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "ZID verify returned success=false".to_string(),
            }),
        )
            .into_response();
    }

    let zid_user_id = zid_identity.user_id.clone();
    let zid_username = zid_identity.username.clone();

    if zid_user_id.is_empty() || zid_username.is_empty() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "ZID verify returned empty user_id/username".to_string(),
            }),
        )
            .into_response();
    }

    // 1) try find linked user_id by (provider, subject=zid_user_id)
    let maybe_user_id = match state
        .external_identity_repo
        .find_user_id("zid", &zid_user_id)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("Failed to lookup external identity: {}", e),
                }),
            )
                .into_response();
        }
    };

    // 2) resolve/create user
    let user_id = if let Some(user_id) = maybe_user_id {
        user_id
    } else {
        // no link yet -> provision user by username (unique)
        // Если username уже похож на email — используем его, иначе генерируем placeholder
        let email = if zid_username.contains('@') {
            zid_username.clone()
        } else {
            format!("{}@zid.local", zid_username)
        };

        // Try to reuse existing user by username (if already provisioned but link missing)
        let existing = match state.user_repo.find_by_username(&zid_username).await {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: format!("Failed to find user by username: {}", e),
                    }),
                )
                    .into_response();
            }
        };

        let user = if let Some(u) = existing {
            u
        } else {
            match state.user_repo.create_sso_user(&email, &zid_username).await {
                Ok(u) => u,
                Err(e) => {
                    return (
                        StatusCode::BAD_GATEWAY,
                        Json(ErrorResponse {
                            error: format!("Failed to create SSO user: {}", e),
                        }),
                    )
                        .into_response();
                }
            }
        };

        // Link identity
        if let Err(e) = state
            .external_identity_repo
            .link_user(user.id, "zid", &zid_user_id)
            .await
        {
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("Failed to link external identity: {}", e),
                }),
            )
                .into_response();
        }

        user.id
    };

    // 3) create session JWT + set httpOnly cookie
    let session_token = match state.auth_service.create_session_token(user_id) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to create session token: {}", e),
                }),
            )
                .into_response();
        }
    };

    let secure = config.zid.callback_url.starts_with("https://");
    let set_session_cookie = format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400{secure_flag}",
        name = CFGY_SESSION_COOKIE_NAME,
        value = session_token,
        secure_flag = if secure { "; Secure" } else { "" }
    );

    // clear state cookie (prevent replay)
    let clear_state_cookie = format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure_flag}",
        name = ZID_STATE_COOKIE_NAME,
        secure_flag = if secure { "; Secure" } else { "" }
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

    (out_headers, Redirect::temporary(&next)).into_response()
}

/// GET /api/auth/me
///
/// Возвращает текущего пользователя.
/// Для Web UI — работает через httpOnly cookie `cfgy_session` (см. extract_user()).
/// Для CLI/API — также поддерживает Authorization header (fallback).
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

/// POST /api/auth/logout
///
/// Очищает httpOnly cookie веб-сессии (`cfgy_session`).
pub async fn logout() -> impl IntoResponse {
    let config = Config::load().unwrap_or_default();
    let secure = config.zid.callback_url.starts_with("https://");

    let clear_session_cookie = format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure_flag}",
        name = CFGY_SESSION_COOKIE_NAME,
        secure_flag = if secure { "; Secure" } else { "" }
    );

    let mut out_headers = HeaderMap::new();
    out_headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_session_cookie).unwrap(),
    );

    (out_headers, StatusCode::NO_CONTENT)
}

/// POST /api/auth/login
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

    // Создать JWT сессионный токен
    let session_token = state.auth_service.create_session_token(user.id).ok(); // Если не удалось создать, продолжаем без токена

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
