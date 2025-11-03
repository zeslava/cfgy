use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use super::types::{AccessToken, ConfigKey, Environment, Project, User};

/// Репозиторий для работы с пользователями
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Создать нового пользователя
    async fn create(&self, email: &str, password_hash: &str) -> Result<User>;

    /// Найти пользователя по email
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;

    /// Найти пользователя по ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
}

/// Репозиторий для работы с проектами
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Создать новый проект
    async fn create(&self, name: &str, description: Option<&str>, owner_id: Uuid) -> Result<Project>;

    /// Найти проект по ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>>;

    /// Найти все проекты пользователя
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Project>>;

    /// Удалить проект
    async fn delete(&self, id: Uuid) -> Result<()>;

    /// Обновить проект
    async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>) -> Result<Project>;
}

/// Репозиторий для работы с окружениями
#[async_trait]
pub trait EnvironmentRepository: Send + Sync {
    /// Создать новое окружение
    async fn create(&self, project_id: Uuid, name: &str) -> Result<Environment>;

    /// Найти окружение по ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Environment>>;

    /// Найти все окружения проекта
    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<Environment>>;

    /// Найти окружение по проекту и имени
    async fn find_by_project_and_name(&self, project_id: Uuid, name: &str) -> Result<Option<Environment>>;

    /// Удалить окружение
    async fn delete(&self, id: Uuid) -> Result<()>;
}

/// Репозиторий для работы с конфигурационными ключами
#[async_trait]
pub trait ConfigKeyRepository: Send + Sync {
    /// Создать или обновить конфигурационный ключ
    async fn upsert(&self, environment_id: Uuid, key: &str, encrypted_value: &[u8]) -> Result<ConfigKey>;

    /// Найти конфигурационный ключ
    async fn find(&self, environment_id: Uuid, key: &str) -> Result<Option<ConfigKey>>;

    /// Получить все ключи окружения
    async fn find_all(&self, environment_id: Uuid) -> Result<Vec<ConfigKey>>;

    /// Удалить конфигурационный ключ
    async fn delete(&self, environment_id: Uuid, key: &str) -> Result<bool>;

    /// Удалить все ключи окружения
    async fn delete_all(&self, environment_id: Uuid) -> Result<()>;
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
