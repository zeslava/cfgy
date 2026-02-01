// Smoke-тесты для проверки всех сценариев работы CFGY API
// Запуск: cargo test -p cfgy-tests
#![allow(dead_code, unused_imports)]

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

// Конфигурация тестов
const BASE_URL: &str = "http://localhost:8080";
const TIMEOUT_SEC: u64 = 30;

#[derive(Debug, Serialize, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    user_id: Uuid,
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateProjectRequest {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateProjectRequest {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectResponse {
    id: Uuid,
    name: String,
    description: Option<String>,
    owner_id: Uuid,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateEnvironmentRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EnvironmentResponse {
    id: Uuid,
    project_id: Uuid,
    name: String,
    created_at: String,
    updated_at: String,
}


/// Тестовый клиент для работы с API
struct TestClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl TestClient {
    fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(TIMEOUT_SEC))
            .build()
            .expect("Не удалось создать HTTP клиент");
        
        Self {
            client,
            base_url: base_url.to_string(),
            token: None,
        }
    }

    fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {}", t))
    }

    /// Проверка доступности сервера
    async fn check_server_available(&self) -> Result<()> {
        // Попробуем выполнить простой GET запрос к корневому пути или API
        let url = format!("{}/api/projects", self.base_url);
        match self.client.get(&url).send().await {
            Ok(_) => Ok(()), // Даже если вернет ошибку аутентификации, сервер работает
            Err(e) if e.is_connect() => {
                anyhow::bail!(
                    "Сервер недоступен на {}. Убедитесь, что сервер запущен:\n\
                    - PostgreSQL: task up\n\
                    - Миграции: task migrate\n\
                    - Сервер: task run (или вручную с переменными окружения)",
                    self.base_url
                );
            }
            Err(e) => Err(e).context("Ошибка подключения к серверу"),
        }
    }

    /// Регистрация пользователя
    async fn register(&self, email: &str, password: &str) -> Result<()> {
        let url = format!("{}/api/auth/register", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&RegisterRequest {
                email: email.to_string(),
                password: password.to_string(),
            })
            .send()
            .await
            .with_context(|| {
                format!(
                    "Не удалось отправить запрос регистрации. Проверьте, что сервер запущен на {}",
                    self.base_url
                )
            })?;

        let status = response.status();
        if status.is_success() || status.as_u16() == 400 {
            // 400 может быть если пользователь уже существует - это нормально для тестов
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка регистрации ({}): {}", status, error_text);
        }
    }

    /// Вход пользователя
    async fn login(&self, email: &str, password: &str) -> Result<LoginResponse> {
        let url = format!("{}/api/auth/login", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&LoginRequest {
                email: email.to_string(),
                password: password.to_string(),
            })
            .send()
            .await
            .context("Не удалось отправить запрос входа")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка входа ({}): {}", status, error_text);
        }

        let login_response: LoginResponse = response
            .json()
            .await
            .context("Не удалось распарсить ответ входа")?;

        Ok(login_response)
    }

    /// Создание проекта
    async fn create_project(&self, name: &str, description: Option<&str>) -> Result<ProjectResponse> {
        let url = format!("{}/api/projects", self.base_url);
        let mut request = self.client.post(&url);

        if let Some(token) = &self.auth_header() {
            request = request.header("Authorization", token);
        }

        let response = request
            .json(&CreateProjectRequest {
                name: name.to_string(),
                description: description.map(|s| s.to_string()),
            })
            .send()
            .await
            .context("Не удалось отправить запрос создания проекта")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка создания проекта ({}): {}", status, error_text);
        }

        let project: ProjectResponse = response
            .json()
            .await
            .context("Не удалось распарсить ответ создания проекта")?;

        Ok(project)
    }

    /// Получение проекта
    async fn get_project(&self, project_id: &Uuid) -> Result<ProjectResponse> {
        let url = format!("{}/api/projects/{}", self.base_url, project_id);
        let mut request = self.client.get(&url);

        if let Some(token) = &self.auth_header() {
            request = request.header("Authorization", token);
        }

        let response = request
            .send()
            .await
            .context("Не удалось отправить запрос получения проекта")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка получения проекта ({}): {}", status, error_text);
        }

        let project: ProjectResponse = response
            .json()
            .await
            .context("Не удалось распарсить ответ получения проекта")?;

        Ok(project)
    }

    /// Список проектов
    async fn list_projects(&self) -> Result<Vec<ProjectResponse>> {
        let url = format!("{}/api/projects", self.base_url);
        let mut request = self.client.get(&url);

        if let Some(token) = &self.auth_header() {
            request = request.header("Authorization", token);
        }

        let response = request
            .send()
            .await
            .context("Не удалось отправить запрос списка проектов")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка получения списка проектов ({}): {}", status, error_text);
        }

        let projects: Vec<ProjectResponse> = response
            .json()
            .await
            .context("Не удалось распарсить ответ списка проектов")?;

        Ok(projects)
    }

    /// Обновление проекта
    async fn update_project(
        &self,
        project_id: &Uuid,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<ProjectResponse> {
        let url = format!("{}/api/projects/{}", self.base_url, project_id);
        let mut request = self.client.patch(&url);

        if let Some(token) = &self.auth_header() {
            request = request.header("Authorization", token);
        }

        let response = request
            .json(&UpdateProjectRequest {
                name: name.map(|s| s.to_string()),
                description: description.map(|s| s.to_string()),
            })
            .send()
            .await
            .context("Не удалось отправить запрос обновления проекта")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка обновления проекта ({}): {}", status, error_text);
        }

        let project: ProjectResponse = response
            .json()
            .await
            .context("Не удалось распарсить ответ обновления проекта")?;

        Ok(project)
    }

    /// Удаление проекта
    async fn delete_project(&self, project_id: &Uuid) -> Result<()> {
        let url = format!("{}/api/projects/{}", self.base_url, project_id);
        let mut request = self.client.delete(&url);

        if let Some(token) = &self.auth_header() {
            request = request.header("Authorization", token);
        }

        let response = request
            .send()
            .await
            .context("Не удалось отправить запрос удаления проекта")?;

        let status = response.status();
        if !status.is_success() && status != reqwest::StatusCode::NO_CONTENT {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка удаления проекта ({}): {}", status, error_text);
        }

        Ok(())
    }

    /// Создание окружения
    async fn create_environment(&self, project_id: &Uuid, name: &str) -> Result<EnvironmentResponse> {
        let url = format!("{}/api/projects/{}/environments", self.base_url, project_id);
        let mut request = self.client.post(&url);

        if let Some(token) = &self.auth_header() {
            request = request.header("Authorization", token);
        }

        let response = request
            .json(&CreateEnvironmentRequest {
                name: name.to_string(),
            })
            .send()
            .await
            .context("Не удалось отправить запрос создания окружения")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка создания окружения ({}): {}", status, error_text);
        }

        let env: EnvironmentResponse = response
            .json()
            .await
            .context("Не удалось распарсить ответ создания окружения")?;

        Ok(env)
    }

    /// Список окружений
    async fn list_environments(&self, project_id: &Uuid) -> Result<Vec<EnvironmentResponse>> {
        let url = format!("{}/api/projects/{}/environments", self.base_url, project_id);
        let mut request = self.client.get(&url);

        if let Some(token) = &self.auth_header() {
            request = request.header("Authorization", token);
        }

        let response = request
            .send()
            .await
            .context("Не удалось отправить запрос списка окружений")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка получения списка окружений ({}): {}", status, error_text);
        }

        let environments: Vec<EnvironmentResponse> = response
            .json()
            .await
            .context("Не удалось распарсить ответ списка окружений")?;

        Ok(environments)
    }

    /// Установка секретов
    async fn set_secrets(&self, project_id: &Uuid, env_name: &str, secrets: &serde_json::Value) -> Result<()> {
        let url = format!("{}/api/projects/{}/secrets/{}", self.base_url, project_id, env_name);
        let mut request = self.client.put(&url);

        if let Some(token) = &self.auth_header() {
            request = request.header("Authorization", token);
        }

        let response = request
            .json(secrets)
            .send()
            .await
            .context("Не удалось отправить запрос установки секретов")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка установки секретов ({}): {}", status, error_text);
        }

        Ok(())
    }

    /// Получение секретов
    async fn get_secrets(&self, project_id: &Uuid, env_name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/projects/{}/secrets/{}", self.base_url, project_id, env_name);
        let mut request = self.client.get(&url);

        if let Some(token) = &self.auth_header() {
            request = request.header("Authorization", token);
        }

        let response = request
            .send()
            .await
            .context("Не удалось отправить запрос получения секретов")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка получения секретов ({}): {}", status, error_text);
        }

        let secrets: serde_json::Value = response
            .json()
            .await
            .context("Не удалось распарсить ответ получения секретов")?;

        Ok(secrets)
    }

    /// Удаление секрета
    async fn delete_secret(&self, project_id: &Uuid, env_name: &str, key: &str) -> Result<()> {
        let encoded_key = urlencoding::encode(key);
        let url = format!("{}/api/projects/{}/secrets/{}/{}", self.base_url, project_id, env_name, encoded_key);
        let mut request = self.client.delete(&url);

        if let Some(token) = &self.auth_header() {
            request = request.header("Authorization", token);
        }

        let response = request
            .send()
            .await
            .context("Не удалось отправить запрос удаления секрета")?;

        let status = response.status();
        if !status.is_success() && status != reqwest::StatusCode::NO_CONTENT {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка удаления секрета ({}): {}", status, error_text);
        }

        Ok(())
    }
}

/// Генерация уникального email для тестов
fn test_email() -> String {
    format!("test-{}@example.com", uuid::Uuid::new_v4())
}

/// Генерация уникального имени проекта
fn test_project_name() -> String {
    format!("test-project-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap())
}

// ========== ТЕСТЫ ==========

#[tokio::test]
async fn test_auth_register_and_login() -> Result<()> {
    println!("\n=== Тест: Регистрация и вход ===");
    
    let client = TestClient::new(BASE_URL);
    
    // Проверка доступности сервера
    client.check_server_available().await?;
    println!("✓ Сервер доступен");
    
    let email = test_email();
    let password = "testpass123";

    // Регистрация
    client.register(&email, password).await?;
    println!("✓ Регистрация прошла успешно");

    // Вход
    let login_response = client.login(&email, password).await?;
    assert_eq!(login_response.email, email);
    assert!(login_response.session_token.is_some());
    println!("✓ Вход прошел успешно, получен токен");

    Ok(())
}

#[tokio::test]
async fn test_project_crud() -> Result<()> {
    println!("\n=== Тест: CRUD операции с проектами ===");
    
    let mut client = TestClient::new(BASE_URL);
    
    // Проверка доступности сервера
    client.check_server_available().await?;
    println!("✓ Сервер доступен");
    
    let email = test_email();
    let password = "testpass123";

    // Регистрация и вход
    client.register(&email, password).await?;
    let login_response = client.login(&email, password).await?;
    client.set_token(login_response.session_token.unwrap());

    // 1. Создание проекта
    let project_name = test_project_name();
    let project_description = "Тестовый проект для проверки CRUD";
    
    let project = client
        .create_project(&project_name, Some(project_description))
        .await?;
    
    assert_eq!(project.name, project_name);
    assert_eq!(project.description.as_deref(), Some(project_description));
    println!("✓ Проект создан: {}", project.id);

    let project_id = project.id;

    // 2. Получение проекта
    let retrieved_project = client.get_project(&project_id).await?;
    assert_eq!(retrieved_project.id, project_id);
    assert_eq!(retrieved_project.name, project_name);
    println!("✓ Проект получен по ID");

    // 3. Список проектов
    let projects = client.list_projects().await?;
    assert!(projects.iter().any(|p| p.id == project_id));
    println!("✓ Проект найден в списке (всего проектов: {})", projects.len());

    // 4. Обновление проекта (только имя)
    let new_name = format!("{}-renamed", project_name);
    let updated_project = client
        .update_project(&project_id, Some(&new_name), None)
        .await?;
    
    assert_eq!(updated_project.name, new_name);
    assert_eq!(updated_project.description.as_deref(), Some(project_description));
    println!("✓ Проект обновлен (имя изменено)");

    // 5. Обновление проекта (имя и описание)
    let new_name2 = format!("{}-edited", project_name);
    let new_description = "Обновленное описание проекта";
    let updated_project2 = client
        .update_project(&project_id, Some(&new_name2), Some(new_description))
        .await?;
    
    assert_eq!(updated_project2.name, new_name2);
    assert_eq!(updated_project2.description.as_deref(), Some(new_description));
    println!("✓ Проект обновлен (имя и описание изменены)");

    // 6. Удаление проекта
    client.delete_project(&project_id).await?;
    println!("✓ Проект удален");

    // 7. Проверка, что проект удален
    let get_result = client.get_project(&project_id).await;
    assert!(get_result.is_err());
    println!("✓ Подтверждено, что проект больше не существует");

    Ok(())
}

#[tokio::test]
async fn test_environments() -> Result<()> {
    println!("\n=== Тест: Управление окружениями ===");
    
    let mut client = TestClient::new(BASE_URL);
    
    // Проверка доступности сервера
    client.check_server_available().await?;
    println!("✓ Сервер доступен");
    
    let email = test_email();
    let password = "testpass123";

    // Регистрация и вход
    client.register(&email, password).await?;
    let login_response = client.login(&email, password).await?;
    client.set_token(login_response.session_token.unwrap());

    // Создание проекта
    let project_name = test_project_name();
    let project = client
        .create_project(&project_name, Some("Проект для тестирования окружений"))
        .await?;
    let project_id = project.id;
    println!("✓ Проект создан: {}", project_id);

    // Создание окружений
    let env1_name = "development";
    let env1 = client.create_environment(&project_id, env1_name).await?;
    assert_eq!(env1.name, env1_name);
    println!("✓ Окружение '{}' создано", env1_name);

    let env2_name = "production";
    let env2 = client.create_environment(&project_id, env2_name).await?;
    assert_eq!(env2.name, env2_name);
    println!("✓ Окружение '{}' создано", env2_name);

    // Список окружений
    let environments = client.list_environments(&project_id).await?;
    assert_eq!(environments.len(), 2);
    assert!(environments.iter().any(|e| e.name == env1_name));
    assert!(environments.iter().any(|e| e.name == env2_name));
    println!("✓ Получен список окружений (всего: {})", environments.len());

    // Очистка
    client.delete_project(&project_id).await?;
    println!("✓ Тестовый проект удален");

    Ok(())
}

#[tokio::test]
async fn test_secrets_crud() -> Result<()> {
    println!("\n=== Тест: CRUD операции с секретами ===");
    
    let mut client = TestClient::new(BASE_URL);
    
    // Проверка доступности сервера
    client.check_server_available().await?;
    println!("✓ Сервер доступен");
    
    let email = test_email();
    let password = "testpass123";

    // Регистрация и вход
    client.register(&email, password).await?;
    let login_response = client.login(&email, password).await?;
    client.set_token(login_response.session_token.unwrap());

    // Создание проекта и окружения
    let project_name = test_project_name();
    let project = client
        .create_project(&project_name, None)
        .await?;
    let project_id = project.id;
    
    let env_name = "development";
    let _env = client.create_environment(&project_id, env_name).await?;
    println!("✓ Проект и окружение созданы");

    // 1. Установка секретов
    let secrets = json!({
        "DATABASE_URL": "postgres://localhost:5432/test",
        "API_KEY": "secret-key-123",
        "DEBUG": "true"
    });

    client.set_secrets(&project_id, env_name, &secrets).await?;
    println!("✓ Секреты установлены");

    // 2. Получение секретов
    let retrieved_secrets = client.get_secrets(&project_id, env_name).await?;
    assert_eq!(retrieved_secrets["DATABASE_URL"], "postgres://localhost:5432/test");
    assert_eq!(retrieved_secrets["API_KEY"], "secret-key-123");
    assert_eq!(retrieved_secrets["DEBUG"], "true");
    println!("✓ Секреты получены и проверены");

    // 3. Обновление секретов (добавление новых)
    let updated_secrets = json!({
        "DATABASE_URL": "postgres://localhost:5432/test",
        "API_KEY": "secret-key-123",
        "DEBUG": "false",  // изменено
        "NEW_KEY": "new-value"  // новый секрет
    });

    client.set_secrets(&project_id, env_name, &updated_secrets).await?;
    
    let retrieved_secrets2 = client.get_secrets(&project_id, env_name).await?;
    assert_eq!(retrieved_secrets2["DEBUG"], "false");
    assert_eq!(retrieved_secrets2["NEW_KEY"], "new-value");
    println!("✓ Секреты обновлены (DEBUG изменен, NEW_KEY добавлен)");

    // 4. Удаление секрета
    client.delete_secret(&project_id, env_name, "NEW_KEY").await?;
    
    let retrieved_secrets3 = client.get_secrets(&project_id, env_name).await?;
    assert!(!retrieved_secrets3.as_object().unwrap().contains_key("NEW_KEY"));
    assert_eq!(retrieved_secrets3["DATABASE_URL"], "postgres://localhost:5432/test");
    println!("✓ Секрет NEW_KEY удален, остальные секреты сохранены");

    // Очистка
    client.delete_project(&project_id).await?;
    println!("✓ Тестовый проект удален");

    Ok(())
}

#[tokio::test]
async fn test_project_update_partial() -> Result<()> {
    println!("\n=== Тест: Частичное обновление проекта ===");
    
    let mut client = TestClient::new(BASE_URL);
    
    // Проверка доступности сервера
    client.check_server_available().await?;
    println!("✓ Сервер доступен");
    
    let email = test_email();
    let password = "testpass123";

    // Регистрация и вход
    client.register(&email, password).await?;
    let login_response = client.login(&email, password).await?;
    client.set_token(login_response.session_token.unwrap());

    // Создание проекта
    let project_name = test_project_name();
    let initial_description = "Начальное описание";
    let project = client
        .create_project(&project_name, Some(initial_description))
        .await?;
    let project_id = project.id;
    println!("✓ Проект создан с описанием");

    // Обновление только описания (имя не меняем)
    let new_description = "Только описание изменено";
    let updated_project = client
        .update_project(&project_id, None, Some(new_description))
        .await?;
    
    assert_eq!(updated_project.name, project_name); // имя не изменилось
    assert_eq!(updated_project.description.as_deref(), Some(new_description));
    println!("✓ Обновлено только описание, имя осталось прежним");

    // Обновление только имени (описание не меняем)
    let new_name = format!("{}-name-only", project_name);
    let updated_project2 = client
        .update_project(&project_id, Some(&new_name), None)
        .await?;
    
    assert_eq!(updated_project2.name, new_name); // имя изменилось
    assert_eq!(updated_project2.description.as_deref(), Some(new_description)); // описание не изменилось
    println!("✓ Обновлено только имя, описание осталось прежним");

    // Очистка
    client.delete_project(&project_id).await?;
    println!("✓ Тестовый проект удален");

    Ok(())
}

#[tokio::test]
async fn test_secrets_with_special_characters() -> Result<()> {
    println!("\n=== Тест: Секреты со специальными символами ===");
    
    let mut client = TestClient::new(BASE_URL);
    
    // Проверка доступности сервера
    client.check_server_available().await?;
    println!("✓ Сервер доступен");
    
    let email = test_email();
    let password = "testpass123";

    // Регистрация и вход
    client.register(&email, password).await?;
    let login_response = client.login(&email, password).await?;
    client.set_token(login_response.session_token.unwrap());

    // Создание проекта и окружения
    let project_name = test_project_name();
    let project = client.create_project(&project_name, None).await?;
    let project_id = project.id;
    
    let env_name = "development";
    let _env = client.create_environment(&project_id, env_name).await?;

    // Секреты со специальными символами
    let secrets = json!({
        "KEY_WITH_SLASH": "value/with/slashes",
        "KEY_WITH_SPECIAL": "value with spaces & symbols!@#",
        "KEY_WITH_EQUALS": "value=with=equals",
        "KEY_WITH_NEWLINE": "value\nwith\nnewlines"
    });

    client.set_secrets(&project_id, env_name, &secrets).await?;
    println!("✓ Секреты со специальными символами установлены");

    let retrieved = client.get_secrets(&project_id, env_name).await?;
    assert_eq!(retrieved["KEY_WITH_SLASH"], "value/with/slashes");
    assert_eq!(retrieved["KEY_WITH_SPECIAL"], "value with spaces & symbols!@#");
    assert_eq!(retrieved["KEY_WITH_EQUALS"], "value=with=equals");
    assert_eq!(retrieved["KEY_WITH_NEWLINE"], "value\nwith\nnewlines");
    println!("✓ Все секреты со специальными символами корректно сохранены и получены");

    // Тест удаления секрета со специальными символами в ключе
    let special_key = "KEY_WITH/SPECIAL/CHARS";
    let secrets_special = json!({
        special_key: "value"
    });
    client.set_secrets(&project_id, env_name, &secrets_special).await?;
    client.delete_secret(&project_id, env_name, special_key).await?;
    println!("✓ Секрет с специальными символами в ключе удален");

    // Очистка
    client.delete_project(&project_id).await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_environments_secrets() -> Result<()> {
    println!("\n=== Тест: Секреты в нескольких окружениях ===");
    
    let mut client = TestClient::new(BASE_URL);
    
    // Проверка доступности сервера
    client.check_server_available().await?;
    println!("✓ Сервер доступен");
    
    let email = test_email();
    let password = "testpass123";

    // Регистрация и вход
    client.register(&email, password).await?;
    let login_response = client.login(&email, password).await?;
    client.set_token(login_response.session_token.unwrap());

    // Создание проекта
    let project_name = test_project_name();
    let project = client.create_project(&project_name, None).await?;
    let project_id = project.id;
    
    // Создание двух окружений
    let _dev_env = client.create_environment(&project_id, "development").await?;
    let _prod_env = client.create_environment(&project_id, "production").await?;

    // Установка разных секретов для каждого окружения
    let dev_secrets = json!({
        "DATABASE_URL": "postgres://localhost:5432/dev_db",
        "DEBUG": "true"
    });

    let prod_secrets = json!({
        "DATABASE_URL": "postgres://prod-db.example.com:5432/prod_db",
        "DEBUG": "false"
    });

    client.set_secrets(&project_id, "development", &dev_secrets).await?;
    client.set_secrets(&project_id, "production", &prod_secrets).await?;
    println!("✓ Секреты установлены для обоих окружений");

    // Проверка изоляции окружений
    let dev_retrieved = client.get_secrets(&project_id, "development").await?;
    let prod_retrieved = client.get_secrets(&project_id, "production").await?;

    assert_eq!(dev_retrieved["DATABASE_URL"], "postgres://localhost:5432/dev_db");
    assert_eq!(dev_retrieved["DEBUG"], "true");
    
    assert_eq!(prod_retrieved["DATABASE_URL"], "postgres://prod-db.example.com:5432/prod_db");
    assert_eq!(prod_retrieved["DEBUG"], "false");

    println!("✓ Секреты изолированы между окружениями");

    // Очистка
    client.delete_project(&project_id).await?;

    Ok(())
}

fn main() {
    println!("Smoke-тесты для CFGY API");
    println!("Убедитесь, что сервер запущен на {}", BASE_URL);
    println!("\nДля запуска сервера выполните:");
    println!("  1. task up  # Запустить PostgreSQL в Docker");
    println!("  2. task migrate  # Применить миграции");
    println!("  3. task run  # Запустить сервер (или вручную с переменными окружения)");
    println!("\nЗапустите тесты командой: cargo test --bin smoke_tests\n");
}

