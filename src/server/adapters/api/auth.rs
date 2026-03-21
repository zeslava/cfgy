use axum::http::{HeaderMap, StatusCode, header};
use uuid::Uuid;

use crate::server::adapters::api::handlers::{AppState, ErrorResponse};

/// Имя httpOnly cookie, в котором хранится JWT сессионный токен для Web UI
const CFGY_SESSION_COOKIE_NAME: &str = "cfgy_session";

/// Структура для извлечения аутентифицированного пользователя из запроса
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

fn get_cookie_value(cookie_header: &str, name: &str) -> Option<String> {
    for part in cookie_header.split(';') {
        let p = part.trim();
        if let Some(rest) = p.strip_prefix(&format!("{}=", name))
            && !rest.is_empty()
        {
            return Some(rest.to_string());
        }
    }
    None
}

/// Извлечь аутентифицированного пользователя из запроса
///
/// Поддерживаем два способа:
/// 1) Web UI: httpOnly cookie `cfgy_session` (JWT)
/// 2) CLI/API: `Authorization: Bearer ...` (JWT session или API token)
pub async fn extract_user(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, (StatusCode, axum::Json<ErrorResponse>)> {
    // 1) Попробовать сессионный JWT из cookie (httpOnly)
    if let Some(cookie_header) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        if let Some(session_token) = get_cookie_value(cookie_header, CFGY_SESSION_COOKIE_NAME) {
            match state.auth_service.verify_session_token(&session_token) {
                Ok(session_claims) => {
                    return Ok(AuthenticatedUser {
                        user_id: session_claims.sub,
                    });
                }
                Err(e) => {
                    tracing::warn!("JWT cookie session verification failed: {}", e);
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        axum::Json(ErrorResponse {
                            error: format!(
                                "Неверная или истекшая сессия. Пожалуйста, войдите заново: {}",
                                e
                            ),
                        }),
                    ));
                }
            }
        }
    }

    // 2) Fallback: Authorization header (как раньше)
    let auth_header = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(ErrorResponse {
                    error: "Требуется заголовок Authorization или cookie cfgy_session".to_string(),
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
                        error: format!(
                            "Неверный или истекший сессионный токен. Пожалуйста, войдите заново: {}",
                            e
                        ),
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
