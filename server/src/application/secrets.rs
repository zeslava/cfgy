use anyhow::{Context, Result};
use std::collections::HashMap;
use uuid::Uuid;

use crate::ports::types::ConfigKey;
use crate::ports::{ConfigKeyRepository, CryptoService, EnvironmentRepository, ProjectRepository};

/// Сервис для работы с секретами
pub struct SecretService<CKR, ER, PR, CS>
where
    CKR: ConfigKeyRepository,
    ER: EnvironmentRepository,
    PR: ProjectRepository,
    CS: CryptoService,
{
    config_key_repo: CKR,
    env_repo: ER,
    project_repo: PR,
    crypto_service: CS,
}

impl<CKR, ER, PR, CS> SecretService<CKR, ER, PR, CS>
where
    CKR: ConfigKeyRepository + Clone,
    ER: EnvironmentRepository + Clone,
    PR: ProjectRepository + Clone,
    CS: CryptoService + Clone,
{
    /// Создать новый сервис
    pub fn new(config_key_repo: CKR, env_repo: ER, project_repo: PR, crypto_service: CS) -> Self {
        Self {
            config_key_repo,
            env_repo,
            project_repo,
            crypto_service,
        }
    }

    /// Установить или обновить секрет
    pub async fn set_secret(
        &self,
        project_id: Uuid,
        env_name: &str,
        owner_id: Uuid,
        key: &str,
        value: &str,
    ) -> Result<ConfigKey> {
        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Получить или создать окружение
        let env = self
            .get_or_create_environment(project_id, env_name, owner_id)
            .await?;

        // Валидация ключа
        if key.is_empty() {
            anyhow::bail!("Ключ не может быть пустым");
        }
        if key.len() > 255 {
            anyhow::bail!("Ключ слишком длинный (максимум 255 символов)");
        }

        // Зашифровать значение
        let encrypted_value = self
            .crypto_service
            .encrypt(value)
            .await
            .context("Не удалось зашифровать значение")?;

        // Сохранить в БД
        self.config_key_repo
            .upsert(env.id, key, &encrypted_value)
            .await
            .context("Не удалось сохранить секрет")
    }

    /// Получить секрет
    pub async fn get_secret(
        &self,
        project_id: Uuid,
        env_name: &str,
        owner_id: Uuid,
        key: &str,
    ) -> Result<String> {
        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Получить окружение
        let env = self
            .env_repo
            .find_by_project_and_name(project_id, env_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Окружение '{}' не найдено", env_name))?;

        // Получить зашифрованный секрет
        let config_key = self
            .config_key_repo
            .find(env.id, key)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Секрет '{}' не найден", key))?;

        // Расшифровать значение
        self.crypto_service
            .decrypt(&config_key.encrypted_value)
            .await
            .context("Не удалось расшифровать значение")
    }

    /// Получить все секреты окружения
    pub async fn list_secrets(
        &self,
        project_id: Uuid,
        env_name: &str,
        owner_id: Uuid,
    ) -> Result<HashMap<String, String>> {
        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Получить окружение
        let env = self
            .env_repo
            .find_by_project_and_name(project_id, env_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Окружение '{}' не найдено", env_name))?;

        // Получить все зашифрованные секреты
        let config_keys = self
            .config_key_repo
            .find_all(env.id)
            .await
            .context("Не удалось получить секреты")?;

        // Сохранить количество секретов для проверки
        let total_secrets = config_keys.len();

        // Расшифровать все секреты
        let mut secrets = HashMap::new();
        let mut errors = Vec::new();

        for config_key in config_keys {
            match self
                .crypto_service
                .decrypt(&config_key.encrypted_value)
                .await
            {
                Ok(value) => {
                    secrets.insert(config_key.key.clone(), value);
                }
                Err(e) => {
                    // Диагностика: вывести hex dump первых байт для анализа
                    let hex_preview = config_key
                        .encrypted_value
                        .iter()
                        .take(32)
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let hex_dump = if config_key.encrypted_value.len() > 32 {
                        format!("{}...", hex_preview)
                    } else {
                        hex_preview
                    };

                    let error_msg = format!(
                        "Не удалось расшифровать секрет '{}': {} (размер данных: {} байт, hex: {})",
                        config_key.key,
                        e,
                        config_key.encrypted_value.len(),
                        hex_dump
                    );
                    tracing::error!("{}", error_msg);
                    errors.push(error_msg);
                    // Пропускаем этот секрет, но продолжаем обработку остальных
                }
            }
        }

        // Если не удалось расшифровать ни одного секрета, но они есть
        if secrets.is_empty() && total_secrets > 0 {
            anyhow::bail!(
                "Не удалось расшифровать ни одного секрета. Возможно, изменился CFGY_MASTER_KEY или данные повреждены.\n\
                Убедитесь, что используется тот же CFGY_MASTER_KEY, который применялся при шифровании секретов.\n\
                Если ключ изменился, секреты необходимо перешифровать. Ошибки: {}",
                errors.join("; ")
            );
        }

        // Если были ошибки для некоторых секретов, но хотя бы один расшифрован
        if !errors.is_empty() {
            tracing::warn!(
                "Частичная расшифровка секретов: успешно расшифровано {}/{}, ошибки: {}",
                secrets.len(),
                secrets.len() + errors.len(),
                errors.join("; ")
            );
        }

        Ok(secrets)
    }

    /// Установить несколько секретов сразу
    pub async fn set_secrets(
        &self,
        project_id: Uuid,
        env_name: &str,
        owner_id: Uuid,
        secrets: &HashMap<String, String>,
    ) -> Result<Vec<ConfigKey>> {
        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Получить или создать окружение
        let env = self
            .get_or_create_environment(project_id, env_name, owner_id)
            .await?;

        // Сохранить все секреты
        let mut saved_keys = Vec::new();
        for (key, value) in secrets {
            // Валидация ключа
            if key.is_empty() {
                anyhow::bail!("Ключ не может быть пустым");
            }
            if key.len() > 255 {
                anyhow::bail!("Ключ слишком длинный (максимум 255 символов)");
            }

            // Зашифровать значение
            let encrypted_value = self
                .crypto_service
                .encrypt(value)
                .await
                .context(format!("Не удалось зашифровать секрет '{}'", key))?;

            // Сохранить в БД
            let config_key = self
                .config_key_repo
                .upsert(env.id, key, &encrypted_value)
                .await
                .context(format!("Не удалось сохранить секрет '{}'", key))?;

            saved_keys.push(config_key);
        }

        Ok(saved_keys)
    }

    /// Удалить секрет
    pub async fn delete_secret(
        &self,
        project_id: Uuid,
        env_name: &str,
        owner_id: Uuid,
        key: &str,
    ) -> Result<bool> {
        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Получить окружение
        let env = self
            .env_repo
            .find_by_project_and_name(project_id, env_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Окружение '{}' не найдено", env_name))?;

        self.config_key_repo.delete(env.id, key).await
    }

    /// Проверить права доступа к проекту
    async fn check_access(&self, project_id: Uuid, user_id: Uuid) -> Result<()> {
        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Проект не найден"))?;

        if project.owner_id != user_id {
            anyhow::bail!("У вас нет прав доступа к этому проекту");
        }

        Ok(())
    }

    /// Получить или создать окружение
    async fn get_or_create_environment(
        &self,
        project_id: Uuid,
        env_name: &str,
        owner_id: Uuid,
    ) -> Result<crate::ports::types::Environment> {
        // Попробовать найти существующее окружение
        if let Some(env) = self
            .env_repo
            .find_by_project_and_name(project_id, env_name)
            .await?
        {
            return Ok(env);
        }

        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Создать новое окружение
        self.env_repo
            .create(project_id, env_name)
            .await
            .context("Не удалось создать окружение")
    }
}
