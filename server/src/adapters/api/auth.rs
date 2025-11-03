use axum::http::{HeaderMap, StatusCode};
use uuid::Uuid;

use crate::adapters::api::handlers::{AppState, ErrorResponse};

/// Структура для извлечения аутентифицированного пользователя из запроса
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

/// Извлечь аутентифицированного пользователя из заголовков запроса
pub async fn extract_user(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, (StatusCode, axum::Json<ErrorResponse>)> {
    // Получить заголовок Authorization
    let auth_header = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(ErrorResponse {
                    error: "Требуется заголовок Authorization".to_string(),
                }),
            )
        })?;

    // Извлечь токен из заголовка (формат: "Bearer <token>" или просто "<token>")
    let auth_str = auth_header.to_str().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(ErrorResponse {
                error: "Неверный формат заголовка Authorization".to_string(),
            }),
        )
    })?;

    let token = if auth_str.starts_with("Bearer ") {
        &auth_str[7..]
    } else {
        auth_str
    };

    // Проверить, выглядит ли токен как JWT (3 части, разделенные точками)
    let is_jwt_format = token.split('.').count() == 3;

    // Сначала попробуем проверить как JWT сессионный токен
    // Если токен выглядит как JWT, но не проходит проверку, вернем понятную ошибку
    if is_jwt_format {
        match state.auth_service.verify_session_token(token) {
            Ok(session_claims) => {
                return Ok(AuthenticatedUser {
                    user_id: session_claims.sub,
                });
            }
            Err(e) => {
                tracing::warn!("JWT token verification failed: {}", e);
                return Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(ErrorResponse {
                        error: format!("Неверный или истекший сессионный токен. Пожалуйста, войдите заново: {}", e),
                    }),
                ));
            }
        }
    }

    // Если токен не выглядит как JWT, проверим как обычный API токен
    let access_token = state
        .auth_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::warn!("API token verification failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(AuthenticatedUser {
        user_id: access_token.user_id,
    })
}

