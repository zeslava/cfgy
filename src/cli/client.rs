use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Клиент для работы с CFGY API
pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

#[derive(Serialize)]
struct CreateProjectRequest {
    name: String,
    description: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ApiClient {
    /// Создать новый клиент API
    pub fn new(base_url: String, token: Option<String>) -> Self {
        // Добавить протокол, если его нет
        let base_url = if base_url.starts_with("http://") || base_url.starts_with("https://") {
            base_url.trim_end_matches('/').to_string()
        } else {
            format!("http://{}", base_url.trim_end_matches('/'))
        };

        Self {
            client: Client::new(),
            base_url,
            token,
        }
    }

    /// Получить заголовки для запросов
    fn headers(&self) -> Result<reqwest::header::HeaderMap> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::CONTENT_TYPE, "application/json".parse()?);

        if let Some(token) = &self.token {
            let auth_value = format!("Bearer {}", token);
            headers.insert(reqwest::header::AUTHORIZATION, auth_value.parse()?);
        }

        Ok(headers)
    }

    /// Создать проект
    pub async fn create_project(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<ProjectResponse> {
        let url = format!("{}/api/projects", self.base_url);
        let request = CreateProjectRequest { name, description };

        let response = self
            .client
            .post(&url)
            .headers(self.headers()?)
            .json(&request)
            .send()
            .await
            .context("Не удалось отправить запрос на создание проекта")?;

        let status = response.status();
        if status.is_success() {
            let project: ProjectResponse = response
                .json()
                .await
                .context("Не удалось распарсить ответ создания проекта")?;
            Ok(project)
        } else {
            let error: ErrorResponse = response.json().await.unwrap_or_else(|_| ErrorResponse {
                error: format!("HTTP {}", status),
            });
            anyhow::bail!("Ошибка создания проекта: {}", error.error);
        }
    }

    /// Получить список проектов
    pub async fn list_projects(&self) -> Result<Vec<ProjectResponse>> {
        let url = format!("{}/api/projects", self.base_url);

        let response = self
            .client
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await
            .context("Не удалось отправить запрос на получение списка проектов")?;

        let status = response.status();
        if status.is_success() {
            let projects: Vec<ProjectResponse> = response
                .json()
                .await
                .context("Не удалось распарсить ответ списка проектов")?;
            Ok(projects)
        } else {
            let error: ErrorResponse = response.json().await.unwrap_or_else(|_| ErrorResponse {
                error: format!("HTTP {}", status),
            });
            anyhow::bail!("Ошибка получения списка проектов: {}", error.error);
        }
    }

    /// Получить проект по ID
    #[allow(dead_code)]
    pub async fn get_project(&self, id: Uuid) -> Result<ProjectResponse> {
        let url = format!("{}/api/projects/{}", self.base_url, id);

        let response = self
            .client
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await
            .context("Не удалось отправить запрос на получение проекта")?;

        let status = response.status();
        if status.is_success() {
            let project: ProjectResponse = response
                .json()
                .await
                .context("Не удалось распарсить ответ получения проекта")?;
            Ok(project)
        } else {
            let error: ErrorResponse = response.json().await.unwrap_or_else(|_| ErrorResponse {
                error: format!("HTTP {}", status),
            });
            anyhow::bail!("Ошибка получения проекта: {}", error.error);
        }
    }

    /// Найти проект по имени
    pub async fn find_project_by_name(&self, name: &str) -> Result<ProjectResponse> {
        let projects = self.list_projects().await?;
        projects
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow::anyhow!("Проект '{}' не найден", name))
    }

    /// Обновить проект
    pub async fn update_project(
        &self,
        project_id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<ProjectResponse> {
        let url = format!("{}/api/projects/{}", self.base_url, project_id);

        #[derive(serde::Serialize)]
        struct UpdateRequest {
            name: Option<String>,
            description: Option<String>,
        }

        let request = UpdateRequest { name, description };

        let response = self
            .client
            .patch(&url)
            .headers(self.headers()?)
            .json(&request)
            .send()
            .await
            .context("Не удалось отправить запрос на обновление проекта")?;

        let status = response.status();
        if status.is_success() {
            let project: ProjectResponse = response
                .json()
                .await
                .context("Не удалось распарсить ответ обновления проекта")?;
            Ok(project)
        } else {
            let error: ErrorResponse = response.json().await.unwrap_or_else(|_| ErrorResponse {
                error: format!("HTTP {}", status),
            });
            anyhow::bail!("Ошибка обновления проекта: {}", error.error);
        }
    }

    /// Удалить проект
    pub async fn delete_project(&self, project_id: Uuid) -> Result<()> {
        let url = format!("{}/api/projects/{}", self.base_url, project_id);

        let response = self
            .client
            .delete(&url)
            .headers(self.headers()?)
            .send()
            .await
            .context("Не удалось отправить запрос на удаление проекта")?;

        let status = response.status();
        if status.is_success() || status == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let error: ErrorResponse = response.json().await.unwrap_or_else(|_| ErrorResponse {
                error: format!("HTTP {}", status),
            });
            anyhow::bail!("Ошибка удаления проекта: {}", error.error);
        }
    }

    /// Установить секреты для окружения
    pub async fn set_secrets(
        &self,
        project_id: Uuid,
        env_name: &str,
        secrets: &HashMap<String, String>,
    ) -> Result<()> {
        let url = format!(
            "{}/api/projects/{}/configurations/{}/parameters",
            self.base_url, project_id, env_name
        );

        let response = self
            .client
            .put(&url)
            .headers(self.headers()?)
            .json(secrets)
            .send()
            .await
            .context("Не удалось отправить запрос на установку секретов")?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let error: ErrorResponse = response.json().await.unwrap_or_else(|_| ErrorResponse {
                error: format!("HTTP {}", status),
            });
            anyhow::bail!("Ошибка установки секретов: {}", error.error);
        }
    }

    /// Получить секреты для окружения
    pub async fn get_secrets(
        &self,
        project_id: Uuid,
        env_name: &str,
    ) -> Result<HashMap<String, String>> {
        let url = format!(
            "{}/api/projects/{}/configurations/{}/parameters",
            self.base_url, project_id, env_name
        );

        let response = self
            .client
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await
            .context("Не удалось отправить запрос на получение секретов")?;

        let status = response.status();
        if status.is_success() {
            let secrets: HashMap<String, String> = response
                .json()
                .await
                .context("Не удалось распарсить ответ получения секретов")?;
            Ok(secrets)
        } else {
            let error: ErrorResponse = response.json().await.unwrap_or_else(|_| ErrorResponse {
                error: format!("HTTP {}", status),
            });
            anyhow::bail!("Ошибка получения секретов: {}", error.error);
        }
    }

    /// Удалить секрет для окружения
    pub async fn delete_secret(&self, project_id: Uuid, env_name: &str, key: &str) -> Result<()> {
        // Используем path параметр для ключа
        let url = format!(
            "{}/api/projects/{}/configurations/{}/parameters/{}",
            self.base_url,
            project_id,
            env_name,
            urlencoding::encode(key)
        );

        let response = self
            .client
            .delete(&url)
            .headers(self.headers()?)
            .send()
            .await
            .context("Не удалось отправить запрос на удаление секрета")?;

        let status = response.status();
        if status.is_success() || status == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let error: ErrorResponse = response.json().await.unwrap_or_else(|_| ErrorResponse {
                error: format!("HTTP {}", status),
            });
            anyhow::bail!("Ошибка удаления секрета: {}", error.error);
        }
    }
}
