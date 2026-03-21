use anyhow::{Context, Result};
use std::collections::HashMap;
use uuid::Uuid;

use crate::server::ports::entities::ConfigKey;
use crate::server::ports::{
    ConfigKeyRepository, ConfigurationRepository, CryptoService, ProjectRepository,
};

/// Сервис для работы с параметрами
pub struct ParameterService<CKR, CR, PR, CS>
where
    CKR: ConfigKeyRepository,
    CR: ConfigurationRepository,
    PR: ProjectRepository,
    CS: CryptoService,
{
    config_key_repo: CKR,
    config_repo: CR,
    project_repo: PR,
    crypto_service: CS,
}

impl<CKR, CR, PR, CS> ParameterService<CKR, CR, PR, CS>
where
    CKR: ConfigKeyRepository + Clone,
    CR: ConfigurationRepository + Clone,
    PR: ProjectRepository + Clone,
    CS: CryptoService + Clone,
{
    /// Создать новый сервис
    pub fn new(
        config_key_repo: CKR,
        config_repo: CR,
        project_repo: PR,
        crypto_service: CS,
    ) -> Self {
        Self {
            config_key_repo,
            config_repo,
            project_repo,
            crypto_service,
        }
    }

    /// Установить или обновить параметр
    #[allow(dead_code)]
    pub async fn set_parameter(
        &self,
        project_id: Uuid,
        config_name: &str,
        owner_id: Uuid,
        key: &str,
        value: &str,
    ) -> Result<ConfigKey> {
        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Получить или создать конфигурацию
        let config = self
            .get_or_create_configuration(project_id, config_name, owner_id)
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
            .upsert(config.id, key, &encrypted_value)
            .await
            .context("Не удалось сохранить параметр")
    }

    /// Получить параметр
    #[allow(dead_code)]
    pub async fn get_parameter(
        &self,
        project_id: Uuid,
        config_name: &str,
        owner_id: Uuid,
        key: &str,
    ) -> Result<String> {
        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Получить конфигурацию
        let config = self
            .config_repo
            .find_by_project_and_name(project_id, config_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Конфигурация '{}' не найдена", config_name))?;

        // Получить зашифрованный параметр
        let config_key = self
            .config_key_repo
            .find(config.id, key)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Параметр '{}' не найден", key))?;

        // Расшифровать значение
        self.crypto_service
            .decrypt(&config_key.encrypted_value)
            .await
            .context("Не удалось расшифровать значение")
    }

    /// Получить все параметры конфигурации
    pub async fn list_parameters(
        &self,
        project_id: Uuid,
        config_name: &str,
        owner_id: Uuid,
    ) -> Result<HashMap<String, String>> {
        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Получить конфигурацию
        let config = self
            .config_repo
            .find_by_project_and_name(project_id, config_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Конфигурация '{}' не найдена", config_name))?;

        // Получить все зашифрованные параметры
        let config_keys = self
            .config_key_repo
            .find_all(config.id)
            .await
            .context("Не удалось получить параметры")?;

        // Сохранить количество параметров для проверки
        let total_parameters = config_keys.len();

        // Расшифровать все параметры
        let mut parameters = HashMap::new();
        let mut errors = Vec::new();

        for config_key in config_keys {
            match self
                .crypto_service
                .decrypt(&config_key.encrypted_value)
                .await
            {
                Ok(value) => {
                    parameters.insert(config_key.key.clone(), value);
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
                        "Не удалось расшифровать параметр '{}': {} (размер данных: {} байт, hex: {})",
                        config_key.key,
                        e,
                        config_key.encrypted_value.len(),
                        hex_dump
                    );
                    tracing::error!("{}", error_msg);
                    errors.push(error_msg);
                    // Пропускаем этот параметр, но продолжаем обработку остальных
                }
            }
        }

        // Если не удалось расшифровать ни одного параметра, но они есть
        if parameters.is_empty() && total_parameters > 0 {
            anyhow::bail!(
                "Не удалось расшифровать ни одного параметра. Возможно, изменился CFGY_MASTER_KEY или данные повреждены.\n\
                Убедитесь, что используется тот же CFGY_MASTER_KEY, который применялся при шифровании параметров.\n\
                Если ключ изменился, параметры необходимо перешифровать. Ошибки: {}",
                errors.join("; ")
            );
        }

        // Если были ошибки для некоторых параметров, но хотя бы один расшифрован
        if !errors.is_empty() {
            tracing::warn!(
                "Частичная расшифровка параметров: успешно расшифровано {}/{}, ошибки: {}",
                parameters.len(),
                parameters.len() + errors.len(),
                errors.join("; ")
            );
        }

        Ok(parameters)
    }

    /// Установить несколько параметров сразу
    pub async fn set_parameters(
        &self,
        project_id: Uuid,
        config_name: &str,
        owner_id: Uuid,
        parameters: &HashMap<String, String>,
    ) -> Result<Vec<ConfigKey>> {
        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Получить или создать конфигурацию
        let config = self
            .get_or_create_configuration(project_id, config_name, owner_id)
            .await?;

        // Сохранить все параметры
        let mut saved_keys = Vec::new();
        for (key, value) in parameters {
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
                .context(format!("Не удалось зашифровать параметр '{}'", key))?;

            // Сохранить в БД
            let config_key = self
                .config_key_repo
                .upsert(config.id, key, &encrypted_value)
                .await
                .context(format!("Не удалось сохранить параметр '{}'", key))?;

            saved_keys.push(config_key);
        }

        Ok(saved_keys)
    }

    /// Удалить параметр
    pub async fn delete_parameter(
        &self,
        project_id: Uuid,
        config_name: &str,
        owner_id: Uuid,
        key: &str,
    ) -> Result<bool> {
        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Получить конфигурацию
        let config = self
            .config_repo
            .find_by_project_and_name(project_id, config_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Конфигурация '{}' не найдена", config_name))?;

        self.config_key_repo.delete(config.id, key).await
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

    /// Получить или создать конфигурацию
    async fn get_or_create_configuration(
        &self,
        project_id: Uuid,
        config_name: &str,
        owner_id: Uuid,
    ) -> Result<crate::server::ports::entities::Configuration> {
        // Попробовать найти существующую конфигурацию
        if let Some(config) = self
            .config_repo
            .find_by_project_and_name(project_id, config_name)
            .await?
        {
            return Ok(config);
        }

        // Проверить права доступа
        self.check_access(project_id, owner_id).await?;

        // Создать новую конфигурацию
        self.config_repo
            .create(project_id, config_name)
            .await
            .context("Не удалось создать конфигурацию")
    }
}
