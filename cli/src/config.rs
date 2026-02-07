use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Конфигурация из домашнего файла ~/.cfgy/config.yaml
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HomeConfig {
    /// URL сервера CFGY
    pub server_url: Option<String>,
    /// Токен для аутентификации на сервере (ключ в YAML: token)
    pub token: Option<String>,
}

impl Default for HomeConfig {
    fn default() -> Self {
        Self {
            server_url: None,
            token: None,
        }
    }
}

/// Конфигурация приложения: домашний файл + переменные окружения (env переопределяет файл)
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// API токен для аутентификации на сервере
    pub api_token: Option<String>,
    /// URL сервера CFGY
    pub server_url: String,
    /// Откуда взят токен: "env" или "file" (для вывода в cfgy token)
    pub token_source: Option<&'static str>,
}

/// Путь к файлу конфигурации в домашней директории (~/.cfgy/config.yaml).
/// Только Unix-like (Linux, FreeBSD, macOS).
fn home_config_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|h| h.join(".cfgy").join("config.yaml"))
}

/// Путь к центральному конфигу (для вывода в cfgy config show).
pub fn config_file_path() -> Option<PathBuf> {
    home_config_path()
}

/// Загрузить конфигурацию из ~/.cfgy/config.yaml. При ошибке чтения или парсинга возвращает None.
fn load_home_config() -> Option<HomeConfig> {
    let path = home_config_path()?;
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    serde_yaml::from_str(&content).ok()
}

impl AppConfig {
    /// Загрузить конфигурацию: сначала из ~/.cfgy/config.yaml, затем переопределить из env
    /// (CFGY_SERVER_URL, CFGY_TOKEN). Переменные окружения имеют приоритет.
    pub fn load() -> Self {
        let home = load_home_config();

        let server_url = std::env::var("CFGY_SERVER_URL")
            .ok()
            .or_else(|| home.as_ref().and_then(|h| h.server_url.clone()))
            .unwrap_or_else(|| "127.0.0.1:8080".to_string());

        let api_token_from_env = std::env::var("CFGY_TOKEN").ok();
        let api_token_from_file = home.as_ref().and_then(|h| h.token.clone());

        let (api_token, token_source) = if api_token_from_env.is_some() {
            (api_token_from_env, Some("env"))
        } else if api_token_from_file.is_some() {
            (api_token_from_file, Some("file"))
        } else {
            (None, None)
        };

        Self {
            api_token,
            server_url,
            token_source,
        }
    }

    /// Получить токен API, выдавая ошибку, если он не установлен
    pub fn require_api_token(&self) -> Result<String> {
        self.api_token.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "Для работы с сервером необходимо установить API токен:\n\
                export CFGY_TOKEN=<ваш_токен>\n\
                или укажите token в ~/.cfgy/config.yaml\n\
                Токен можно создать в веб-дашборде."
            )
        })
    }

    /// Получить URL сервера, выдавая ошибку, если он не установлен
    pub fn require_server_url(&self) -> Result<String> {
        Ok(self.server_url.clone())
    }

    /// Проверить, настроен ли доступ к серверу (есть и токен, и URL)
    #[allow(dead_code)]
    pub fn has_server_access(&self) -> bool {
        self.api_token.is_some() && !self.server_url.is_empty()
    }
}

/// Интерактивная настройка: запросить server_url и token, записать в ~/.cfgy/config.yaml.
/// Создаёт директорию ~/.cfgy при необходимости, выставляет права 0o600 на файл.
pub fn interactive_setup(current: &AppConfig) -> Result<()> {
    let path = home_config_path().ok_or_else(|| anyhow::anyhow!("Переменная HOME не задана"))?;

    println!("Конфиг ~/.cfgy/config.yaml отсутствует или неполный.");
    println!("Введите данные для подключения к серверу CFGY.");
    println!();

    // URL сервера (по умолчанию — текущее значение или 127.0.0.1:8080)
    print!("URL сервера [{}]: ", current.server_url);
    io::stdout().flush().context("Не удалось сбросить stdout")?;
    let mut server_url_input = String::new();
    io::stdin()
        .read_line(&mut server_url_input)
        .context("Не удалось прочитать ввод")?;
    let server_url = server_url_input.trim();
    let server_url = if server_url.is_empty() {
        current.server_url.clone()
    } else {
        server_url.to_string()
    };

    // Токен (обязательно)
    print!("Токен API: ");
    io::stdout().flush().context("Не удалось сбросить stdout")?;
    let mut token_input = String::new();
    io::stdin()
        .read_line(&mut token_input)
        .context("Не удалось прочитать ввод")?;
    let token = token_input.trim().to_string();
    if token.is_empty() {
        anyhow::bail!("Токен не может быть пустым. Создайте токен в веб-дашборде.");
    }

    let home_config = HomeConfig {
        server_url: Some(server_url.clone()),
        token: Some(token),
    };
    let content =
        serde_yaml::to_string(&home_config).context("Не удалось сериализовать конфиг в YAML")?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Не удалось создать директорию: {}", parent.display()))?;
    }

    fs::write(&path, content)
        .with_context(|| format!("Не удалось записать файл: {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)
            .with_context(|| format!("Не удалось прочитать метаданные: {}", path.display()))?
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)
            .with_context(|| format!("Не удалось установить права на файл: {}", path.display()))?;
    }

    println!("✓ Конфиг сохранён: {}", path.display());
    Ok(())
}
