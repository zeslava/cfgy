use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapters::api::auth::extract_user;
use crate::application::{auth::AuthService, projects::ProjectService, secrets::SecretService};

/// Контекст приложения (все сервисы и репозитории)
#[derive(Clone)]
pub struct AppState {
    pub project_service: Arc<ProjectService<
        crate::adapters::persistence::PostgresRepositories,
        crate::adapters::persistence::PostgresRepositories,
        crate::adapters::persistence::PostgresRepositories,
        crate::adapters::crypto::AesCryptoService,
    >>,
    pub secret_service: Arc<SecretService<
        crate::adapters::persistence::PostgresRepositories,
        crate::adapters::persistence::PostgresRepositories,
        crate::adapters::persistence::PostgresRepositories,
        crate::adapters::crypto::AesCryptoService,
    >>,
    pub auth_service: Arc<AuthService<
        crate::adapters::persistence::PostgresRepositories,
        crate::adapters::persistence::PostgresRepositories,
    >>,
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
pub struct EnvironmentResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct CreateEnvironmentRequest {
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

#[derive(Deserialize)]
pub struct SetSecretsRequest {
    #[serde(flatten)]
    pub secrets: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Параметры пути для DELETE секрета
/// Извлекается из URL: /api/projects/:id/secrets/:env/:key
/// Имена полей структуры должны совпадать с именами параметров в URL
#[derive(Deserialize)]
pub struct DeleteSecretPath {
    pub id: String,
    pub env: String,
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
    let session_token = state
        .auth_service
        .create_session_token(user.id)
        .ok(); // Если не удалось создать, продолжаем без токена

    Ok((
        StatusCode::CREATED,
        Json(LoginResponse {
            user_id: user.id,
            email: user.email,
            session_token,
        }),
    ))
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
    let session_token = state
        .auth_service
        .create_session_token(user.id)
        .ok(); // Если не удалось создать, продолжаем без токена

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
    let project = state
        .project_service
        .get_project(id)
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
        .update_project(id, user.user_id, request.name.as_deref(), request.description.as_deref())
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

/// POST /api/projects/:id/environments
pub async fn create_environment(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<CreateEnvironmentRequest>,
) -> Result<(StatusCode, Json<EnvironmentResponse>), (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    let env = state
        .project_service
        .create_environment(project_id, user.user_id, &request.name)
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
        Json(EnvironmentResponse {
            id: env.id,
            project_id: env.project_id,
            name: env.name,
            created_at: env.created_at,
            updated_at: env.updated_at,
        }),
    ))
}

/// GET /api/projects/:id/environments
pub async fn list_environments(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Vec<EnvironmentResponse>>, (StatusCode, Json<ErrorResponse>)> {
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

    let envs = state
        .project_service
        .list_environments(project_id)
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
        envs.into_iter()
            .map(|e| EnvironmentResponse {
                id: e.id,
                project_id: e.project_id,
                name: e.name,
                created_at: e.created_at,
                updated_at: e.updated_at,
            })
            .collect(),
    ))
}

/// DELETE /api/projects/:id/environments/:env_id
pub async fn delete_environment(
    State(state): State<AppState>,
    Path((project_id, env_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    
    state
        .project_service
        .delete_environment(env_id, user.user_id)
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

/// PUT /api/projects/:id/secrets/:env
pub async fn set_secrets(
    State(state): State<AppState>,
    Path((project_id, env_name)): Path<(Uuid, String)>,
    headers: HeaderMap,
    Json(request): Json<SetSecretsRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    state
        .secret_service
        .set_secrets(project_id, &env_name, user.user_id, &request.secrets)
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

/// GET /api/projects/:id/secrets/:env
pub async fn get_secrets(
    State(state): State<AppState>,
    Path((project_id, env_name)): Path<(Uuid, String)>,
    headers: HeaderMap,
) -> Result<Json<HashMap<String, String>>, (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    let secrets = state
        .secret_service
        .list_secrets(project_id, &env_name, user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(secrets))
}

/// DELETE /api/projects/:id/secrets/:env/:key
pub async fn delete_secret(
    State(state): State<AppState>,
    Path(params): Path<DeleteSecretPath>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Преобразуем project_id из String в Uuid
    let project_id = params.id
        .parse::<Uuid>()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Неверный формат UUID проекта: {}", params.id),
                }),
            )
        })?;
    let user = extract_user(&headers, &state).await?;
    let deleted = state
        .secret_service
        .delete_secret(project_id, &params.env, user.user_id, &params.key)
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
                error: format!("Секрет '{}' не найден в окружении '{}'", params.key, params.env),
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
