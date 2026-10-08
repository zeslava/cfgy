use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::server::ports::entities::{AccessToken, ConfigKey, Configuration, Project, User};
use crate::server::ports::{
    AccessTokenRepository, ConfigKeyRepository, ConfigurationRepository,
    ExternalIdentityRepository, ProjectRepository, UserRepository,
};

/// Структура, объединяющая все репозитории Postgres
#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    /// Создать новую структуру репозиториев
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, email: &str, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash)
            VALUES ($1, $2)
            RETURNING id, email, username, password_hash, created_at, updated_at
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .context("Не удалось создать пользователя")?;

        Ok(user)
    }

    async fn create_sso_user(&self, email: &str, username: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, username, password_hash)
            VALUES ($1, $2, NULL)
            RETURNING id, email, username, password_hash, created_at, updated_at
            "#,
        )
        .bind(email)
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .context("Не удалось создать SSO пользователя")?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, username, password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти пользователя по email")?;

        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, username, password_hash, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти пользователя по username")?;

        Ok(user)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, username, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти пользователя по ID")?;

        Ok(user)
    }

    async fn count(&self) -> Result<i64> {
        sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .context("Не удалось посчитать пользователей")
    }

    async fn update_profile(&self, id: Uuid, email: &str, username: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET email = $1, username = $2, updated_at = now()
            WHERE id = $3
            RETURNING id, email, username, password_hash, created_at, updated_at
            "#,
        )
        .bind(email)
        .bind(username)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Не удалось обновить профиль пользователя")?;

        Ok(user)
    }
}

#[async_trait]
impl ExternalIdentityRepository for PostgresUserRepository {
    async fn find_user_id(&self, provider: &str, subject: &str) -> Result<Option<Uuid>> {
        let row = sqlx::query_as::<_, (Uuid,)>(
            r#"
            SELECT user_id
            FROM external_identities
            WHERE provider = $1 AND subject = $2
            "#,
        )
        .bind(provider)
        .bind(subject)
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти пользователя по external identity")?;

        Ok(row.map(|(user_id,)| user_id))
    }

    async fn link_user(&self, user_id: Uuid, provider: &str, subject: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO external_identities (user_id, provider, subject)
            VALUES ($1, $2, $3)
            ON CONFLICT (provider, subject) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .bind(subject)
        .execute(&self.pool)
        .await
        .context("Не удалось привязать external identity к пользователю")?;

        Ok(())
    }
}

#[async_trait]
impl ProjectRepository for PostgresUserRepository {
    async fn create(
        &self,
        name: &str,
        description: Option<&str>,
        owner_id: Uuid,
    ) -> Result<Project> {
        let project = sqlx::query_as::<_, Project>(
            r#"
            INSERT INTO projects (name, description, owner_id)
            VALUES ($1, $2, $3)
            RETURNING id, name, description, owner_id, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await
        .context("Не удалось создать проект")?;

        Ok(project)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>> {
        let project = sqlx::query_as::<_, Project>(
            r#"
            SELECT id, name, description, owner_id, created_at, updated_at
            FROM projects
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти проект по ID")?;

        Ok(project)
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Project>> {
        let projects = sqlx::query_as::<_, Project>(
            r#"
            SELECT id, name, description, owner_id, created_at, updated_at
            FROM projects
            WHERE owner_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .context("Не удалось найти проекты пользователя")?;

        Ok(projects)
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Не удалось удалить проект")?;

        Ok(())
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Project> {
        let project = if name.is_some() && description.is_some() {
            sqlx::query_as::<_, Project>(
                r#"
                UPDATE projects
                SET name = $2, description = $3
                WHERE id = $1
                RETURNING id, name, description, owner_id, created_at, updated_at
                "#,
            )
            .bind(id)
            .bind(name)
            .bind(description)
            .fetch_one(&self.pool)
            .await?
        } else if name.is_some() {
            sqlx::query_as::<_, Project>(
                r#"
                UPDATE projects
                SET name = $2
                WHERE id = $1
                RETURNING id, name, description, owner_id, created_at, updated_at
                "#,
            )
            .bind(id)
            .bind(name)
            .fetch_one(&self.pool)
            .await?
        } else if description.is_some() {
            sqlx::query_as::<_, Project>(
                r#"
                UPDATE projects
                SET description = $2
                WHERE id = $1
                RETURNING id, name, description, owner_id, created_at, updated_at
                "#,
            )
            .bind(id)
            .bind(description)
            .fetch_one(&self.pool)
            .await?
        } else {
            anyhow::bail!("Нужно указать хотя бы одно поле для обновления");
        };

        Ok(project)
    }
}

#[async_trait]
impl ConfigurationRepository for PostgresUserRepository {
    async fn create(&self, project_id: Uuid, name: &str) -> Result<Configuration> {
        let config = sqlx::query_as::<_, Configuration>(
            r#"
            INSERT INTO configurations (project_id, name)
            VALUES ($1, $2)
            RETURNING id, project_id, name, created_at, updated_at
            "#,
        )
        .bind(project_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .context("Не удалось создать конфигурацию")?;

        Ok(config)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Configuration>> {
        let config = sqlx::query_as::<_, Configuration>(
            r#"
            SELECT id, project_id, name, created_at, updated_at
            FROM configurations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти конфигурацию по ID")?;

        Ok(config)
    }

    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<Configuration>> {
        let configs = sqlx::query_as::<_, Configuration>(
            r#"
            SELECT id, project_id, name, created_at, updated_at
            FROM configurations
            WHERE project_id = $1
            ORDER BY name
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .context("Не удалось найти конфигурации проекта")?;

        Ok(configs)
    }

    async fn find_by_project_and_name(
        &self,
        project_id: Uuid,
        name: &str,
    ) -> Result<Option<Configuration>> {
        let config = sqlx::query_as::<_, Configuration>(
            r#"
            SELECT id, project_id, name, created_at, updated_at
            FROM configurations
            WHERE project_id = $1 AND name = $2
            "#,
        )
        .bind(project_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти конфигурацию по проекту и имени")?;

        Ok(config)
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM configurations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Не удалось удалить конфигурацию")?;

        Ok(())
    }
}

#[async_trait]
impl ConfigKeyRepository for PostgresUserRepository {
    async fn upsert(
        &self,
        configuration_id: Uuid,
        key: &str,
        encrypted_value: &[u8],
    ) -> Result<ConfigKey> {
        let config_key = sqlx::query_as::<_, ConfigKey>(
            r#"
            INSERT INTO config_keys (configuration_id, key, encrypted_value)
            VALUES ($1, $2, $3)
            ON CONFLICT (configuration_id, key)
            DO UPDATE SET encrypted_value = $3, updated_at = CURRENT_TIMESTAMP
            RETURNING id, configuration_id, key, encrypted_value, created_at, updated_at
            "#,
        )
        .bind(configuration_id)
        .bind(key)
        .bind(encrypted_value)
        .fetch_one(&self.pool)
        .await
        .context("Не удалось создать или обновить параметр")?;

        Ok(config_key)
    }

    async fn find(&self, configuration_id: Uuid, key: &str) -> Result<Option<ConfigKey>> {
        let config_key = sqlx::query_as::<_, ConfigKey>(
            r#"
            SELECT id, configuration_id, key, encrypted_value, created_at, updated_at
            FROM config_keys
            WHERE configuration_id = $1 AND key = $2
            "#,
        )
        .bind(configuration_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти параметр")?;

        Ok(config_key)
    }

    async fn find_all(&self, configuration_id: Uuid) -> Result<Vec<ConfigKey>> {
        let config_keys = sqlx::query_as::<_, ConfigKey>(
            r#"
            SELECT id, configuration_id, key, encrypted_value, created_at, updated_at
            FROM config_keys
            WHERE configuration_id = $1
            ORDER BY key
            "#,
        )
        .bind(configuration_id)
        .fetch_all(&self.pool)
        .await
        .context("Не удалось получить все параметры")?;

        Ok(config_keys)
    }

    async fn delete(&self, configuration_id: Uuid, key: &str) -> Result<bool> {
        let result =
            sqlx::query("DELETE FROM config_keys WHERE configuration_id = $1 AND key = $2")
                .bind(configuration_id)
                .bind(key)
                .execute(&self.pool)
                .await
                .context("Не удалось удалить параметр")?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_all(&self, configuration_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM config_keys WHERE configuration_id = $1")
            .bind(configuration_id)
            .execute(&self.pool)
            .await
            .context("Не удалось удалить все параметры")?;

        Ok(())
    }
}

#[async_trait]
impl AccessTokenRepository for PostgresUserRepository {
    async fn create(
        &self,
        user_id: Uuid,
        project_id: Option<Uuid>,
        name: &str,
        token_hash: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<AccessToken> {
        let token = sqlx::query_as::<_, AccessToken>(
            r#"
            INSERT INTO access_tokens (user_id, project_id, name, token_hash, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, project_id, name, token_hash, expires_at, created_at, last_used_at
            "#,
        )
        .bind(user_id)
        .bind(project_id)
        .bind(name)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            anyhow::anyhow!("Ошибка создания API токена в БД: {} (user_id: {}, name: {})", e, user_id, name)
        })?;

        Ok(token)
    }

    async fn find_by_hash(&self, token_hash: &str) -> Result<Option<AccessToken>> {
        let token = sqlx::query_as::<_, AccessToken>(
            r#"
            SELECT id, user_id, project_id, name, token_hash, expires_at, created_at, last_used_at
            FROM access_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти токен по хэшу")?;

        Ok(token)
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<AccessToken>> {
        let tokens = sqlx::query_as::<_, AccessToken>(
            r#"
            SELECT id, user_id, project_id, name, token_hash, expires_at, created_at, last_used_at
            FROM access_tokens
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .context("Не удалось получить список токенов пользователя")?;

        Ok(tokens)
    }

    async fn update_last_used(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE access_tokens SET last_used_at = CURRENT_TIMESTAMP WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Не удалось обновить время последнего использования токена")?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM access_tokens WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Не удалось удалить токен")?;

        Ok(())
    }
}
