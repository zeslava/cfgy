use anyhow::{Context, Result};
use argon2::Argon2;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::server::config::Config;
use crate::server::ports::entities::{AccessToken, User};
use crate::server::ports::{AccessTokenRepository, UserRepository};

/// JWT Claims для сессионного токена
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionClaims {
    pub sub: Uuid, // User ID
    pub exp: i64,  // Expiration time
    pub iat: i64,  // Issued at
}

/// Сервис для аутентификации и работы с токенами
pub struct AuthService<UR, ATR>
where
    UR: UserRepository,
    ATR: AccessTokenRepository,
{
    user_repo: UR,
    token_repo: ATR,
    jwt_secret: String,
    allow_registration: Option<bool>,
}

/// Регистрация запрещена настройками сервера
#[derive(Debug)]
pub struct RegistrationDisabled;

impl std::fmt::Display for RegistrationDisabled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Регистрация отключена администратором")
    }
}

impl std::error::Error for RegistrationDisabled {}

impl<UR, ATR> AuthService<UR, ATR>
where
    UR: UserRepository,
    ATR: AccessTokenRepository,
{
    /// Создать новый сервис
    pub fn new(user_repo: UR, token_repo: ATR) -> Self {
        let config = Config::load().unwrap_or_default();
        Self {
            user_repo,
            token_repo,
            jwt_secret: config.crypto.jwt_secret,
            allow_registration: config.server.allow_registration,
        }
    }

    /// Зарегистрировать нового пользователя
    pub async fn register(&self, email: &str, password: &str) -> Result<User> {
        let allowed = match self.allow_registration {
            Some(allowed) => allowed,
            None => self.user_repo.count().await? == 0,
        };
        if !allowed {
            return Err(RegistrationDisabled.into());
        }

        // Валидация email
        if email.is_empty() {
            anyhow::bail!("Email не может быть пустым");
        }
        if !email.contains('@') {
            anyhow::bail!("Некорректный формат email");
        }

        // Валидация пароля
        if password.len() < 8 {
            anyhow::bail!("Пароль должен быть не менее 8 символов");
        }

        // Проверить, что пользователь с таким email не существует
        if self.user_repo.find_by_email(email).await?.is_some() {
            anyhow::bail!("Пользователь с таким email уже существует");
        }

        // Хэшировать пароль
        let password_hash = Self::hash_password(password)?;

        // Создать пользователя
        self.user_repo
            .create(email, &password_hash)
            .await
            .context("Не удалось создать пользователя")
    }

    /// Аутентифицировать пользователя по email и паролю
    pub async fn authenticate(&self, email: &str, password: &str) -> Result<User> {
        let user = self
            .user_repo
            .find_by_email(email)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Неверный email или пароль"))?;

        // У SSO-only пользователей password_hash может отсутствовать
        let password_hash = user
            .password_hash
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Неверный email или пароль"))?;

        // Проверить пароль
        Self::verify_password(password, password_hash).context("Неверный email или пароль")?;

        Ok(user)
    }

    /// Создать JWT сессионный токен для пользователя
    pub fn create_session_token(&self, user_id: Uuid) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // Сессия действует 24 часа

        let claims = SessionClaims {
            sub: user_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        let header = Header::default();
        let token = encode(
            &header,
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .context("Не удалось создать JWT токен")?;

        Ok(token)
    }

    /// Проверить JWT сессионный токен
    pub fn verify_session_token(&self, token: &str) -> Result<SessionClaims> {
        use jsonwebtoken::{DecodingKey, Validation, decode};

        let validation = Validation::default();
        let token_data = decode::<SessionClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )
        .context("Неверный или истекший JWT токен")?;

        Ok(token_data.claims)
    }

    /// Создать API токен
    pub async fn create_access_token(
        &self,
        user_id: Uuid,
        project_id: Option<Uuid>,
        name: &str,
        expires_in_days: Option<u32>,
    ) -> Result<(AccessToken, String)> {
        // Сгенерировать случайный токен
        let token = Self::generate_token();

        // Хэшировать токен
        let token_hash = Self::hash_token(&token)?;

        // Вычислить время истечения
        let expires_at =
            expires_in_days.map(|days| chrono::Utc::now() + chrono::Duration::days(days as i64));

        // Сохранить токен в БД
        let access_token = self
            .token_repo
            .create(user_id, project_id, name, &token_hash, expires_at)
            .await
            .context("Не удалось создать API токен")?;

        Ok((access_token, token))
    }

    /// Получить все токены пользователя
    pub async fn list_access_tokens(&self, user_id: Uuid) -> Result<Vec<AccessToken>> {
        self.token_repo
            .find_by_user(user_id)
            .await
            .context("Не удалось получить список токенов")
    }

    /// Удалить токен пользователя
    pub async fn delete_access_token(&self, token_id: Uuid, user_id: Uuid) -> Result<()> {
        // Проверить, что токен принадлежит пользователю
        let tokens = self.list_access_tokens(user_id).await?;
        if !tokens.iter().any(|t| t.id == token_id) {
            anyhow::bail!("Токен не найден или не принадлежит пользователю");
        }

        self.token_repo
            .delete(token_id)
            .await
            .context("Не удалось удалить токен")
    }

    /// Проверить API токен
    pub async fn verify_access_token(&self, token: &str) -> Result<AccessToken> {
        // Хэшировать токен для поиска
        let token_hash = Self::hash_token(token)?;

        let access_token = self
            .token_repo
            .find_by_hash(&token_hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Неверный API токен"))?;

        // Проверить срок действия
        if let Some(expires_at) = access_token.expires_at {
            if chrono::Utc::now() > expires_at {
                anyhow::bail!("API токен истёк");
            }
        }

        // Обновить время последнего использования
        self.token_repo
            .update_last_used(access_token.id)
            .await
            .context("Не удалось обновить время использования токена")?;

        Ok(access_token)
    }

    /// Хэшировать пароль
    fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Ошибка хэширования пароля: {}", e))?;
        Ok(password_hash.to_string())
    }

    /// Проверить пароль
    fn verify_password(password: &str, password_hash: &str) -> Result<()> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| anyhow::anyhow!("Ошибка парсинга хэша пароля: {}", e))?;
        let argon2 = Argon2::default();
        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|e| anyhow::anyhow!("Ошибка проверки пароля: {}", e))?;
        Ok(())
    }

    /// Сгенерировать случайный токен
    fn generate_token() -> String {
        use ring::rand::{SecureRandom, SystemRandom};
        let rng = SystemRandom::new();
        let mut bytes = [0u8; 32];
        rng.fill(&mut bytes)
            .expect("Не удалось сгенерировать токен");
        hex::encode(bytes)
    }

    /// Хэшировать токен
    fn hash_token(token: &str) -> Result<String> {
        use ring::digest;
        let mut context = digest::Context::new(&digest::SHA256);
        context.update(token.as_bytes());
        Ok(hex::encode(context.finish().as_ref()))
    }
}
