use anyhow::{Context, Result};
use uuid::Uuid;

use crate::ports::entities::{Configuration, Project};
use crate::ports::{
    ConfigKeyRepository, ConfigurationRepository, CryptoService, ProjectRepository,
};

/// Сервис для работы с проектами
pub struct ProjectService<PR, CR, CKR, CS>
where
    PR: ProjectRepository,
    CR: ConfigurationRepository,
    CKR: ConfigKeyRepository,
    CS: CryptoService,
{
    project_repo: PR,
    config_repo: CR,
    config_key_repo: CKR,
    crypto_service: CS,
}

impl<PR, CR, CKR, CS> ProjectService<PR, CR, CKR, CS>
where
    PR: ProjectRepository,
    CR: ConfigurationRepository,
    CKR: ConfigKeyRepository,
    CS: CryptoService,
{
    /// Создать новый сервис
    pub fn new(
        project_repo: PR,
        config_repo: CR,
        config_key_repo: CKR,
        crypto_service: CS,
    ) -> Self {
        Self {
            project_repo,
            config_repo,
            config_key_repo,
            crypto_service,
        }
    }

    /// Создать новый проект
    pub async fn create_project(
        &self,
        name: &str,
        description: Option<&str>,
        owner_id: Uuid,
    ) -> Result<Project> {
        // Валидация имени проекта
        if name.is_empty() {
            anyhow::bail!("Имя проекта не может быть пустым");
        }

        if name.len() > 255 {
            anyhow::bail!("Имя проекта слишком длинное (максимум 255 символов)");
        }

        self.project_repo
            .create(name, description, owner_id)
            .await
            .context("Не удалось создать проект")
    }

    /// Получить проект по ID
    pub async fn get_project(&self, id: Uuid) -> Result<Project> {
        self.project_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Проект не найден"))
    }

    /// Получить все проекты пользователя
    pub async fn list_projects(&self, owner_id: Uuid) -> Result<Vec<Project>> {
        self.project_repo
            .find_by_owner(owner_id)
            .await
            .context("Не удалось получить список проектов")
    }

    /// Удалить проект
    pub async fn delete_project(&self, id: Uuid, owner_id: Uuid) -> Result<()> {
        // Проверить, что проект принадлежит пользователю
        let project = self.get_project(id).await?;
        if project.owner_id != owner_id {
            anyhow::bail!("У вас нет прав на удаление этого проекта");
        }

        self.project_repo.delete(id).await
    }

    /// Обновить проект
    pub async fn update_project(
        &self,
        id: Uuid,
        owner_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Project> {
        // Проверить, что проект принадлежит пользователю
        let project = self.get_project(id).await?;
        if project.owner_id != owner_id {
            anyhow::bail!("У вас нет прав на обновление этого проекта");
        }

        // Валидация имени проекта
        if let Some(name) = name {
            if name.is_empty() {
                anyhow::bail!("Имя проекта не может быть пустым");
            }
            if name.len() > 255 {
                anyhow::bail!("Имя проекта слишком длинное (максимум 255 символов)");
            }
        }

        self.project_repo.update(id, name, description).await
    }

    /// Создать конфигурацию в проекте
    pub async fn create_configuration(
        &self,
        project_id: Uuid,
        owner_id: Uuid,
        name: &str,
    ) -> Result<Configuration> {
        // Проверить, что проект принадлежит пользователю
        let _project = self.get_project(project_id).await?;
        if _project.owner_id != owner_id {
            anyhow::bail!("У вас нет прав на создание конфигурации в этом проекте");
        }

        // Валидация имени конфигурации
        if name.is_empty() {
            anyhow::bail!("Имя конфигурации не может быть пустым");
        }

        if name.len() > 255 {
            anyhow::bail!("Имя конфигурации слишком длинное (максимум 255 символов)");
        }

        self.config_repo
            .create(project_id, name)
            .await
            .context("Не удалось создать конфигурацию")
    }

    /// Получить конфигурации проекта
    pub async fn list_configurations(&self, project_id: Uuid) -> Result<Vec<Configuration>> {
        self.config_repo
            .find_by_project(project_id)
            .await
            .context("Не удалось получить список конфигураций")
    }

    /// Удалить конфигурацию
    pub async fn delete_configuration(&self, config_id: Uuid, owner_id: Uuid) -> Result<()> {
        // Проверить, что конфигурация принадлежит проекту пользователя
        let config = self
            .config_repo
            .find_by_id(config_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Конфигурация не найдена"))?;

        let project = self.get_project(config.project_id).await?;
        if project.owner_id != owner_id {
            anyhow::bail!("У вас нет прав на удаление этой конфигурации");
        }

        self.config_repo.delete(config_id).await
    }
}
