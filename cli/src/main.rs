mod client;
mod commands;
mod config;
mod crypto;
mod formats;
mod storage;

use anyhow::{Context, Result};
use clap::Parser;
use client::ApiClient;
use commands::{Cli, Commands, ProjectsAction, TokenAction};
use config::AppConfig;
use formats::OutputFormat;
use std::io::{self, Write};
use std::process::Command;
use storage::ProjectConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Загрузить переменные окружения из .env файла, если он существует
    dotenv::dotenv().ok();

    // Загрузить конфигурацию приложения из переменных окружения
    let app_config = AppConfig::load();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { project } => init_project(project).await,
        Commands::Use { configuration } => use_configuration(configuration).await,
        Commands::Set {
            key,
            value,
            configuration,
        } => set_config(key, value, configuration, &app_config).await,
        Commands::Get {
            key,
            configuration,
            format,
        } => get_config(key, configuration, format, &app_config).await,
        Commands::List {
            configuration,
            format,
        } => list_configs(configuration, format, &app_config).await,
        Commands::Delete { key, configuration } => {
            delete_config(key, configuration, &app_config).await
        }
        Commands::Run { command } => run_command(command, &app_config).await,
        Commands::Projects { action } => handle_projects(action, &app_config).await,
        Commands::Token { action } => handle_token(action, &app_config).await,
        Commands::GenerateSecrets { format } => generate_secrets(format).await,
    }
}

async fn init_project(project_name: Option<String>) -> Result<()> {
    let path = ProjectConfig::config_path_in_current_dir()?;
    let mut config = ProjectConfig::load_or_default(path.clone())?;

    // Имя проекта: из аргумента, из имени директории, или текущее из конфига
    let project_name = project_name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "default".to_string())
    });

    config.init(project_name.clone(), &path)?;
    println!("✓ Проект '{}' инициализирован", project_name);
    println!("  Конфигурация: {}", path.display());
    println!("  Configuration: {}", config.configuration);
    Ok(())
}

async fn set_config(
    key: String,
    value: String,
    configuration: Option<String>,
    app_config: &AppConfig,
) -> Result<()> {
    let config = ProjectConfig::load()?;

    let configuration = configuration.unwrap_or_else(|| config.configuration.clone());

    let token = app_config.require_api_token()?;
    let server_url = app_config.require_server_url()?;

    if config.project.is_empty() {
        anyhow::bail!("Имя проекта не указано в конфигурации. Запустите 'cfgy init'.");
    }

    // Работа с сервером - найти проект по имени
    let client = ApiClient::new(server_url.clone(), Some(token.clone()));
    let project = client.find_project_by_name(&config.project).await?;

    // Получить текущие секреты
    let mut secrets = client
        .get_secrets(project.id, &configuration)
        .await
        .unwrap_or_default();

    // Обновить нужный секрет
    secrets.insert(key.clone(), value.clone());

    // Отправить обновленные секреты
    client
        .set_secrets(project.id, &configuration, &secrets)
        .await?;

    println!(
        "✓ Конфиг '{}' установлен для configuration '{}'",
        key, configuration
    );

    Ok(())
}

async fn get_config(
    key: String,
    configuration: Option<String>,
    format: String,
    app_config: &AppConfig,
) -> Result<()> {
    let config = ProjectConfig::load()?;

    let configuration = configuration.unwrap_or_else(|| config.configuration.clone());

    let token = app_config.require_api_token()?;
    let server_url = app_config.require_server_url()?;

    if config.project.is_empty() {
        anyhow::bail!("Имя проекта не указано в конфигурации. Запустите 'cfgy init'.");
    }

    // Работа с сервером - найти проект по имени
    let client = ApiClient::new(server_url.clone(), Some(token.clone()));
    let project = client.find_project_by_name(&config.project).await?;
    let secrets = client.get_secrets(project.id, &configuration).await?;
    let value = secrets.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!(
            "Конфиг '{}' не найден в configuration '{}'",
            key,
            configuration
        )
    })?;

    let output_format = OutputFormat::from_str(&format)?;

    // Для одного значения создаем HashMap с одним ключом
    let mut single_config = std::collections::HashMap::new();
    single_config.insert(key.clone(), value);

    let output = output_format.output(&single_config)?;
    println!("{}", output);
    Ok(())
}

async fn list_configs(
    configuration: Option<String>,
    format: String,
    app_config: &AppConfig,
) -> Result<()> {
    let config = ProjectConfig::load()?;

    let configuration = configuration.unwrap_or_else(|| config.configuration.clone());

    let token = app_config.require_api_token()?;
    let server_url = app_config.require_server_url()?;

    if config.project.is_empty() {
        anyhow::bail!("Имя проекта не указано в конфигурации. Запустите 'cfgy init'.");
    }

    // Работа с сервером - найти проект по имени
    let client = ApiClient::new(server_url.clone(), Some(token.clone()));
    let project = client.find_project_by_name(&config.project).await?;
    let configs = client.get_secrets(project.id, &configuration).await?;

    if configs.is_empty() {
        eprintln!("⚠ Нет конфигов для configuration '{}'", configuration);
        return Ok(());
    }

    let output_format = OutputFormat::from_str(&format)?;
    let output = output_format.output(&configs)?;
    println!("{}", output);
    Ok(())
}

async fn delete_config(
    key: String,
    configuration: Option<String>,
    app_config: &AppConfig,
) -> Result<()> {
    let config = ProjectConfig::load()?;

    let configuration = configuration.unwrap_or_else(|| config.configuration.clone());

    let token = app_config.require_api_token()?;
    let server_url = app_config.require_server_url()?;

    if config.project.is_empty() {
        anyhow::bail!("Имя проекта не указано в конфигурации. Запустите 'cfgy init'.");
    }

    // Работа с сервером - найти проект по имени
    let client = ApiClient::new(server_url.clone(), Some(token.clone()));
    let project = client.find_project_by_name(&config.project).await?;

    // Удалить секрет на сервере
    client
        .delete_secret(project.id, &configuration, &key)
        .await?;

    println!(
        "✓ Конфиг '{}' удален из configuration '{}'",
        key, configuration
    );

    Ok(())
}

async fn run_command(command: Vec<String>, app_config: &AppConfig) -> Result<()> {
    let config = ProjectConfig::load()?;

    // Получить configuration из конфигурации или из переменной окружения (fallback)
    let configuration = if !config.configuration.is_empty() {
        config.configuration.clone()
    } else {
        std::env::var("CFGY_CONFIGURATION").unwrap_or_else(|_| "dev".to_string())
    };

    let token = app_config.require_api_token()?;
    let server_url = app_config.require_server_url()?;

    if config.project.is_empty() {
        anyhow::bail!("Имя проекта не указано в конфигурации. Запустите 'cfgy init'.");
    }

    // Работа с сервером - найти проект по имени и получить "parameters" для указанной configuration
    let client = ApiClient::new(server_url.clone(), Some(token.clone()));
    let project = client.find_project_by_name(&config.project).await?;
    let configs = client.get_secrets(project.id, &configuration).await?;

    if configs.is_empty() {
        eprintln!("⚠ Нет конфигов для configuration '{}'", configuration);
    }

    let (cmd_name, args) = command
        .split_first()
        .context("Команда не может быть пустой")?;
    let mut child = Command::new(cmd_name)
        .args(args)
        .envs(&configs)
        .spawn()
        .with_context(|| format!("Не удалось запустить команду: {}", cmd_name))?;
    let status = child
        .wait()
        .context("Ошибка при ожидании завершения команды")?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

async fn handle_projects(action: ProjectsAction, app_config: &AppConfig) -> Result<()> {
    match action {
        ProjectsAction::List => list_projects(app_config).await,
        ProjectsAction::Create => create_project(app_config).await,
        ProjectsAction::Delete => delete_project(app_config).await,
        ProjectsAction::Rename => rename_project(app_config).await,
        ProjectsAction::View => view_project(app_config).await,
        ProjectsAction::Edit => edit_project(app_config).await,
    }
}

async fn list_projects(app_config: &AppConfig) -> Result<()> {
    // Получить токен и URL сервера из переменных окружения
    let token = app_config.require_api_token()?;
    let server_url = app_config.require_server_url()?;

    // Работа с сервером - получить список всех проектов
    let client = ApiClient::new(server_url, Some(token.clone()));
    let projects = client.list_projects().await?;

    if projects.is_empty() {
        // Локальная работа
        let config = ProjectConfig::load()?;
        if !config.project.is_empty() {
            println!("✓ Текущий проект: {}", config.project);
        } else {
            println!("⚠ Проект не инициализирован");
        }
        return Ok(());
    }

    // Вывести список проектов
    for project in projects {
        println!("  - {} (ID: {})", project.name, project.id);
    }

    Ok(())
}

async fn create_project(app_config: &AppConfig) -> Result<()> {
    // Получить токен и URL сервера из переменных окружения
    let token = app_config.require_api_token()?;
    let server_url = app_config.require_server_url()?;

    let client = ApiClient::new(server_url.clone(), Some(token.clone()));

    // Запросить имя проекта
    print!("Введите имя проекта: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .context("Не удалось прочитать ввод")?;
    let name = name.trim().to_string();

    if name.is_empty() {
        anyhow::bail!("Имя проекта не может быть пустым");
    }

    let project = client.create_project(name.clone(), None).await?;

    // Сохранить имя проекта в конфигурации
    let path = ProjectConfig::config_path_in_current_dir()?;
    let mut config = ProjectConfig::load_or_default(path.clone())?;
    config.project = name.clone();
    config.save(&path)?;

    println!("✓ Проект '{}' создан (ID: {})", name, project.id);
    println!("  Конфигурация: {}", path.display());
    Ok(())
}

async fn delete_project(app_config: &AppConfig) -> Result<()> {
    // Получить токен и URL сервера из переменных окружения
    let token = app_config.require_api_token()?;
    let server_url = app_config.require_server_url()?;

    let config = ProjectConfig::load()?;
    if config.project.is_empty() {
        anyhow::bail!("Проект не настроен. Используйте 'cfgy init' для инициализации проекта.");
    }
    let project_name = config.project.clone();

    let client = ApiClient::new(server_url.clone(), Some(token.clone()));

    // Найти проект по имени
    let project = client.find_project_by_name(&project_name).await?;

    // Подтверждение удаления
    print!(
        "Вы уверены, что хотите удалить проект '{}'? Это действие нельзя отменить. (yes/no): ",
        project_name
    );
    io::stdout().flush()?;
    let mut confirmation = String::new();
    io::stdin()
        .read_line(&mut confirmation)
        .context("Не удалось прочитать ввод")?;
    let confirmation = confirmation.trim().to_lowercase();

    if confirmation != "yes" {
        println!("Удаление отменено");
        return Ok(());
    }

    // Удалить проект на сервере
    client.delete_project(project.id).await?;

    // Очистить локальную конфигурацию
    let path = ProjectConfig::find_config_path()?;
    let mut config = ProjectConfig::load_or_default(path.clone())?;
    config.project = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "default".to_string());
    config.save(&path)?;

    println!("✓ Проект '{}' удален", project_name);
    Ok(())
}

async fn rename_project(app_config: &AppConfig) -> Result<()> {
    // Получить токен и URL сервера из переменных окружения
    let token = app_config.require_api_token()?;
    let server_url = app_config.require_server_url()?;

    let config = ProjectConfig::load()?;
    if config.project.is_empty() {
        anyhow::bail!("Проект не настроен. Используйте 'cfgy init' для инициализации проекта.");
    }
    let old_name = config.project.clone();

    let client = ApiClient::new(server_url.clone(), Some(token.clone()));

    // Найти проект по имени
    let project = client.find_project_by_name(&old_name).await?;

    // Запросить новое имя проекта
    print!("Введите новое имя проекта: ");
    io::stdout().flush()?;
    let mut new_name = String::new();
    io::stdin()
        .read_line(&mut new_name)
        .context("Не удалось прочитать ввод")?;
    let new_name = new_name.trim().to_string();

    if new_name.is_empty() {
        anyhow::bail!("Имя проекта не может быть пустым");
    }

    if new_name == old_name {
        println!("Имя проекта не изменилось");
        return Ok(());
    }

    // Обновить проект на сервере
    let updated_project = client
        .update_project(project.id, Some(new_name.clone()), None)
        .await?;

    // Обновить локальную конфигурацию
    let path = ProjectConfig::find_config_path()?;
    let mut config = ProjectConfig::load()?;
    config.project = new_name.clone();
    config.save(&path)?;

    println!("✓ Проект переименован: '{}' -> '{}'", old_name, new_name);
    println!("  ID проекта: {}", updated_project.id);
    Ok(())
}

async fn view_project(_app_config: &AppConfig) -> Result<()> {
    let config = ProjectConfig::load()?;
    println!("✓ Информация о проекте:");
    if !config.project.is_empty() {
        println!("  Имя: {}", config.project);
    } else {
        println!("  Имя: не указано");
    }
    println!("  Configuration: {}", config.configuration);
    Ok(())
}

async fn edit_project(app_config: &AppConfig) -> Result<()> {
    // Получить токен и URL сервера из переменных окружения
    let token = app_config.require_api_token()?;
    let server_url = app_config.require_server_url()?;

    let config = ProjectConfig::load()?;
    if config.project.is_empty() {
        anyhow::bail!("Проект не настроен. Используйте 'cfgy init' для инициализации проекта.");
    }
    let project_name = config.project.clone();

    let client = ApiClient::new(server_url.clone(), Some(token.clone()));

    // Найти проект по имени
    let project = client.find_project_by_name(&project_name).await?;

    // Показать текущие данные
    println!("Текущая информация о проекте:");
    println!("  Имя: {}", project.name);
    if let Some(desc) = &project.description {
        println!("  Описание: {}", desc);
    } else {
        println!("  Описание: не указано");
    }
    println!();

    // Запросить новое имя (опционально)
    print!(
        "Новое имя проекта (Enter чтобы оставить '{}'): ",
        project.name
    );
    io::stdout().flush()?;
    let mut new_name_input = String::new();
    io::stdin()
        .read_line(&mut new_name_input)
        .context("Не удалось прочитать ввод")?;
    let new_name = new_name_input.trim();
    let new_name = if new_name.is_empty() {
        None
    } else {
        Some(new_name.to_string())
    };

    // Запросить новое описание (опционально)
    print!("Новое описание проекта (Enter чтобы оставить текущее): ");
    io::stdout().flush()?;
    let mut new_desc_input = String::new();
    io::stdin()
        .read_line(&mut new_desc_input)
        .context("Не удалось прочитать ввод")?;
    let new_desc = new_desc_input.trim();
    let new_desc = if new_desc.is_empty() {
        None
    } else {
        Some(new_desc.to_string())
    };

    // Обновить проект на сервере
    let updated_project = client
        .update_project(project.id, new_name.clone(), new_desc.clone())
        .await?;

    // Обновить локальную конфигурацию, если имя изменилось
    if let Some(new_name) = &new_name {
        let path = ProjectConfig::find_config_path()?;
        let mut config = ProjectConfig::load()?;
        config.project = new_name.clone();
        config.save(&path)?;
    }

    println!("✓ Проект обновлен");
    println!("  Имя: {}", updated_project.name);
    if let Some(desc) = &updated_project.description {
        println!("  Описание: {}", desc);
    }
    Ok(())
}

async fn use_configuration(configuration: String) -> Result<()> {
    let path = ProjectConfig::find_config_path()?;
    let mut config = ProjectConfig::load()?;

    let configuration = configuration.trim().to_string();
    if configuration.is_empty() {
        anyhow::bail!("Configuration не может быть пустой");
    }

    config.configuration = configuration.clone();
    config.save(&path)?;

    println!("✓ Используется configuration '{}'", configuration);
    println!("  Конфигурация: {}", path.display());
    Ok(())
}

/// Обработка команд работы с токенами
async fn handle_token(action: TokenAction, app_config: &AppConfig) -> Result<()> {
    match action {
        TokenAction::List => {
            let env_token = app_config.api_token.as_ref();

            if let Some(token) = env_token {
                println!("✓ Токен установлен");
                let display_len = token.len().min(10);
                println!(
                    "  Первые {} символов: {}...",
                    display_len,
                    &token[..display_len]
                );
                println!("  Источник: переменная окружения CFGY_API_TOKEN");
            } else {
                println!("⚠ Токен не установлен");
                println!("  Установите токен через переменную окружения:");
                println!("    export CFGY_API_TOKEN=<ваш_токен>");
                println!("  Или создайте токен в веб-дашборде");
            }
            Ok(())
        }
    }
}

/// Генерация секретных ключей для сервера
async fn generate_secrets(format: String) -> Result<()> {
    use ring::rand::{SecureRandom, SystemRandom};

    const MASTER_KEY_LEN: usize = 32; // 32 байта = 64 hex символа
    const JWT_SECRET_LEN: usize = 32; // 32 байта = 64 hex символа

    let rng = SystemRandom::new();

    // Генерация master_key (32 байта = 64 hex символа)
    let mut master_key_bytes = vec![0u8; MASTER_KEY_LEN];
    rng.fill(&mut master_key_bytes)
        .context("Не удалось сгенерировать master_key")?;
    let master_key = hex::encode(&master_key_bytes);

    // Генерация jwt_secret (32 байта = 64 hex символа)
    let mut jwt_secret_bytes = vec![0u8; JWT_SECRET_LEN];
    rng.fill(&mut jwt_secret_bytes)
        .context("Не удалось сгенерировать jwt_secret")?;
    let jwt_secret = hex::encode(&jwt_secret_bytes);

    match format.as_str() {
        "env" => {
            println!("# Сгенерированные секреты для .env файла");
            println!("# Скопируйте эти строки в ваш .env файл");
            println!();
            println!("CFGY_MASTER_KEY={}", master_key);
            println!("CFGY_JWT_SECRET={}", jwt_secret);
        }
        "shell" => {
            println!("# Команды для экспорта в shell");
            println!("export CFGY_MASTER_KEY='{}'", master_key);
            println!("export CFGY_JWT_SECRET='{}'", jwt_secret);
        }
        _ => {
            anyhow::bail!("Неизвестный формат: {}. Поддерживаются: env, shell", format);
        }
    }

    println!();
    println!("✓ Секреты сгенерированы успешно");
    println!("  Master Key: {} символов (рекомендуется минимум 32)", master_key.len());
    println!("  JWT Secret: {} символов (рекомендуется минимум 16)", jwt_secret.len());

    Ok(())
}
