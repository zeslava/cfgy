use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use super::entities::{AccessToken, ConfigKey, Configuration, Project, User};

/// Внешняя идентичность пользователя (например, ZID)
#[derive(Debug, Clone)]
pub struct ExternalIdentity {
    pub provider: String,
    pub subject: String,
}

/// Репозиторий для работы с пользователями
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Создать нового пользователя
    async fn create(&self, email: &str, password_hash: &str) -> Result<User>;

    /// Создать нового пользователя без локального пароля (SSO-only).
    ///
    /// Важно: `password_hash` в БД должен быть nullable.
    async fn create_sso_user(&self, email: &str, username: &str) -> Result<User>;

    /// Найти пользователя по email
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;

    /// Найти пользователя по username
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;

    /// Найти пользователя по ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
}

/// Репозиторий для работы с проектами
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Создать новый проект
    async fn create(
        &self,
        name: &str,
        description: Option<&str>,
        owner_id: Uuid,
    ) -> Result<Project>;

    /// Найти проект по ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>>;

    /// Найти все проекты пользователя
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Project>>;

    /// Удалить проект
    async fn delete(&self, id: Uuid) -> Result<()>;

    /// Обновить проект
    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Project>;
}

/// Репозиторий для работы с конфигурациями
#[async_trait]
pub trait ConfigurationRepository: Send + Sync {
    /// Создать новую конфигурацию
    async fn create(&self, project_id: Uuid, name: &str) -> Result<Configuration>;

    /// Найти конфигурацию по ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Configuration>>;

    /// Найти все конфигурации проекта
    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<Configuration>>;

    /// Найти конфигурацию по проекту и имени
    async fn find_by_project_and_name(
        &self,
        project_id: Uuid,
        name: &str,
    ) -> Result<Option<Configuration>>;

    /// Удалить конфигурацию
    async fn delete(&self, id: Uuid) -> Result<()>;
}

/// Репозиторий для работы с параметрами
#[async_trait]
pub trait ConfigKeyRepository: Send + Sync {
    /// Создать или обновить параметр
    async fn upsert(
        &self,
        configuration_id: Uuid,
        key: &str,
        encrypted_value: &[u8],
    ) -> Result<ConfigKey>;

    /// Найти параметр
    async fn find(&self, configuration_id: Uuid, key: &str) -> Result<Option<ConfigKey>>;

    /// Получить все параметры конфигурации
    async fn find_all(&self, configuration_id: Uuid) -> Result<Vec<ConfigKey>>;

    /// Удалить параметр
    async fn delete(&self, configuration_id: Uuid, key: &str) -> Result<bool>;

    /// Удалить все параметры конфигурации
    async fn delete_all(&self, configuration_id: Uuid) -> Result<()>;
}

/// Репозиторий для работы с API токенами
#[async_trait]
pub trait AccessTokenRepository: Send + Sync {
    /// Создать новый API токен
    async fn create(
        &self,
        user_id: Uuid,
        project_id: Option<Uuid>,
        name: &str,
        token_hash: &str,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<AccessToken>;

    /// Найти токен по хэшу
    async fn find_by_hash(&self, token_hash: &str) -> Result<Option<AccessToken>>;

    /// Найти все токены пользователя
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<AccessToken>>;

    /// Обновить время последнего использования токена
    async fn update_last_used(&self, id: Uuid) -> Result<()>;

    /// Удалить токен
    async fn delete(&self, id: Uuid) -> Result<()>;
}

/// Репозиторий для привязки внешних идентичностей (provider+subject -> user_id)
#[async_trait]
pub trait ExternalIdentityRepository: Send + Sync {
    /// Найти пользователя по внешней идентичности (provider + subject)
    async fn find_user_id(&self, provider: &str, subject: &str) -> Result<Option<Uuid>>;

    /// Привязать внешнюю идентичность к пользователю
    async fn link_user(&self, user_id: Uuid, provider: &str, subject: &str) -> Result<()>;
}
