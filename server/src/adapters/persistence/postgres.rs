use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::ports::types::{AccessToken, ConfigKey, Environment, Project, User};
use crate::ports::{
    AccessTokenRepository, ConfigKeyRepository, EnvironmentRepository, ProjectRepository,
    UserRepository,
};

/// Структура, объединяющая все репозитории Postgres
#[derive(Clone)]
pub struct PostgresRepositories {
    pool: PgPool,
}

impl PostgresRepositories {
    /// Создать новую структуру репозиториев
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresRepositories {
    async fn create(&self, email: &str, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, password_hash)
            VALUES ($1, $2)
            RETURNING id, email, password_hash, created_at, updated_at
            "#,
            email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await
        .context("Не удалось создать пользователя")?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти пользователя по email")?;

        Ok(user)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти пользователя по ID")?;

        Ok(user)
    }
}

#[async_trait]
impl ProjectRepository for PostgresRepositories {
    async fn create(&self, name: &str, description: Option<&str>, owner_id: Uuid) -> Result<Project> {
        let project = sqlx::query_as!(
            Project,
            r#"
            INSERT INTO projects (name, description, owner_id)
            VALUES ($1, $2, $3)
            RETURNING id, name, description, owner_id, created_at, updated_at
            "#,
            name,
            description,
            owner_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Не удалось создать проект")?;

        Ok(project)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>> {
        let project = sqlx::query_as!(
            Project,
            r#"
            SELECT id, name, description, owner_id, created_at, updated_at
            FROM projects
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти проект по ID")?;

        Ok(project)
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Project>> {
        let projects = sqlx::query_as!(
            Project,
            r#"
            SELECT id, name, description, owner_id, created_at, updated_at
            FROM projects
            WHERE owner_id = $1
            ORDER BY created_at DESC
            "#,
            owner_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Не удалось найти проекты пользователя")?;

        Ok(projects)
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM projects WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .context("Не удалось удалить проект")?;

        Ok(())
    }

    async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>) -> Result<Project> {
        let project = if name.is_some() && description.is_some() {
            sqlx::query_as!(
                Project,
                r#"
                UPDATE projects
                SET name = $2, description = $3
                WHERE id = $1
                RETURNING id, name, description, owner_id, created_at, updated_at
                "#,
                id,
                name,
                description
            )
            .fetch_one(&self.pool)
            .await?
        } else if name.is_some() {
            sqlx::query_as!(
                Project,
                r#"
                UPDATE projects
                SET name = $2
                WHERE id = $1
                RETURNING id, name, description, owner_id, created_at, updated_at
                "#,
                id,
                name
            )
            .fetch_one(&self.pool)
            .await?
        } else if description.is_some() {
            sqlx::query_as!(
                Project,
                r#"
                UPDATE projects
                SET description = $2
                WHERE id = $1
                RETURNING id, name, description, owner_id, created_at, updated_at
                "#,
                id,
                description
            )
            .fetch_one(&self.pool)
            .await?
        } else {
            anyhow::bail!("Нужно указать хотя бы одно поле для обновления");
        };

        Ok(project)
    }
}

#[async_trait]
impl EnvironmentRepository for PostgresRepositories {
    async fn create(&self, project_id: Uuid, name: &str) -> Result<Environment> {
        let env = sqlx::query_as!(
            Environment,
            r#"
            INSERT INTO environments (project_id, name)
            VALUES ($1, $2)
            RETURNING id, project_id, name, created_at, updated_at
            "#,
            project_id,
            name
        )
        .fetch_one(&self.pool)
        .await
        .context("Не удалось создать окружение")?;

        Ok(env)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Environment>> {
        let env = sqlx::query_as!(
            Environment,
            r#"
            SELECT id, project_id, name, created_at, updated_at
            FROM environments
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти окружение по ID")?;

        Ok(env)
    }

    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<Environment>> {
        let envs = sqlx::query_as!(
            Environment,
            r#"
            SELECT id, project_id, name, created_at, updated_at
            FROM environments
            WHERE project_id = $1
            ORDER BY name
            "#,
            project_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Не удалось найти окружения проекта")?;

        Ok(envs)
    }

    async fn find_by_project_and_name(&self, project_id: Uuid, name: &str) -> Result<Option<Environment>> {
        let env = sqlx::query_as!(
            Environment,
            r#"
            SELECT id, project_id, name, created_at, updated_at
            FROM environments
            WHERE project_id = $1 AND name = $2
            "#,
            project_id,
            name
        )
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти окружение по проекту и имени")?;

        Ok(env)
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM environments WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .context("Не удалось удалить окружение")?;

        Ok(())
    }
}

#[async_trait]
impl ConfigKeyRepository for PostgresRepositories {
    async fn upsert(&self, environment_id: Uuid, key: &str, encrypted_value: &[u8]) -> Result<ConfigKey> {
        let config_key = sqlx::query_as!(
            ConfigKey,
            r#"
            INSERT INTO config_keys (environment_id, key, encrypted_value)
            VALUES ($1, $2, $3)
            ON CONFLICT (environment_id, key)
            DO UPDATE SET encrypted_value = $3, updated_at = CURRENT_TIMESTAMP
            RETURNING id, environment_id, key, encrypted_value, created_at, updated_at
            "#,
            environment_id,
            key,
            encrypted_value as &[u8]
        )
        .fetch_one(&self.pool)
        .await
        .context("Не удалось создать или обновить конфигурационный ключ")?;

        Ok(config_key)
    }

    async fn find(&self, environment_id: Uuid, key: &str) -> Result<Option<ConfigKey>> {
        let config_key = sqlx::query_as!(
            ConfigKey,
            r#"
            SELECT id, environment_id, key, encrypted_value, created_at, updated_at
            FROM config_keys
            WHERE environment_id = $1 AND key = $2
            "#,
            environment_id,
            key
        )
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти конфигурационный ключ")?;

        Ok(config_key)
    }

    async fn find_all(&self, environment_id: Uuid) -> Result<Vec<ConfigKey>> {
        let config_keys = sqlx::query_as!(
            ConfigKey,
            r#"
            SELECT id, environment_id, key, encrypted_value, created_at, updated_at
            FROM config_keys
            WHERE environment_id = $1
            ORDER BY key
            "#,
            environment_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Не удалось получить все конфигурационные ключи")?;

        Ok(config_keys)
    }

    async fn delete(&self, environment_id: Uuid, key: &str) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM config_keys WHERE environment_id = $1 AND key = $2",
            environment_id,
            key
        )
        .execute(&self.pool)
        .await
        .context("Не удалось удалить конфигурационный ключ")?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_all(&self, environment_id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM config_keys WHERE environment_id = $1", environment_id)
            .execute(&self.pool)
            .await
            .context("Не удалось удалить все конфигурационные ключи")?;

        Ok(())
    }
}

#[async_trait]
impl AccessTokenRepository for PostgresRepositories {
    async fn create(
        &self,
        user_id: Uuid,
        project_id: Option<Uuid>,
        name: &str,
        token_hash: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<AccessToken> {
        let token = sqlx::query_as!(
            AccessToken,
            r#"
            INSERT INTO access_tokens (user_id, project_id, name, token_hash, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, project_id, name, token_hash, expires_at, created_at, last_used_at
            "#,
            user_id,
            project_id,
            name,
            token_hash,
            expires_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            anyhow::anyhow!("Ошибка создания API токена в БД: {} (user_id: {}, name: {})", e, user_id, name)
        })?;

        Ok(token)
    }

    async fn find_by_hash(&self, token_hash: &str) -> Result<Option<AccessToken>> {
        let token = sqlx::query_as!(
            AccessToken,
            r#"
            SELECT id, user_id, project_id, name, token_hash, expires_at, created_at, last_used_at
            FROM access_tokens
            WHERE token_hash = $1
            "#,
            token_hash
        )
        .fetch_optional(&self.pool)
        .await
        .context("Не удалось найти токен по хэшу")?;

        Ok(token)
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<AccessToken>> {
        let tokens = sqlx::query_as!(
            AccessToken,
            r#"
            SELECT id, user_id, project_id, name, token_hash, expires_at, created_at, last_used_at
            FROM access_tokens
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Не удалось получить список токенов пользователя")?;

        Ok(tokens)
    }

    async fn update_last_used(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE access_tokens SET last_used_at = CURRENT_TIMESTAMP WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Не удалось обновить время последнего использования токена")?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM access_tokens WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .context("Не удалось удалить токен")?;

        Ok(())
    }
}

