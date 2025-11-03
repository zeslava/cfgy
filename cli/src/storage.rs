use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Конфигурация проекта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Имя проекта (уникально в рамках пользователя)
    #[serde(default = "default_project_name")]
    pub project: String,
    /// Текущее окружение проекта
    #[serde(default = "default_environment")]
    pub environment: String,
}

fn default_project_name() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "test-project-name".to_string())
}

fn default_environment() -> String {
    "dev".to_string()
}

impl ProjectConfig {
    /// Найти путь к конфигурационному файлу
    pub fn find_config_path() -> Result<PathBuf> {
        let current_dir =
            std::env::current_dir().context("Не удалось определить текущую директорию")?;

        let config_path = current_dir.join(".cfgy.yaml");
        if config_path.exists() {
            return Ok(config_path);
        }

        Err(anyhow::anyhow!("Конфигурационный файл не найден"))
    }

    /// Путь к конфигурационному файлу в текущей директории
    pub fn config_path_in_current_dir() -> Result<PathBuf> {
        std::env::current_dir()
            .map(|p| p.join(".cfgy.yaml"))
            .context("Не удалось определить текущую директорию")
    }

    /// Загрузить конфигурацию проекта
    pub fn load() -> Result<Self> {
        let path = Self::find_config_path().or_else(|_| {
            Err(anyhow::anyhow!(
                "Проект не инициализирован. Запустите 'cfgy init' в директории проекта."
            ))
        })?;

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Не удалось прочитать файл: {}", path.display()))?;

        let config: Self = serde_yaml::from_str(&content)
            .with_context(|| format!("Не удалось распарсить YAML: {}", path.display()))?;

        Ok(config)
    }

    /// Загрузить конфигурацию, если она существует, или вернуть дефолт
    pub fn load_or_default(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self {
                project: default_project_name(),
                environment: default_environment(),
            });
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Не удалось прочитать файл: {}", path.display()))?;

        let config: Self = serde_yaml::from_str(&content)
            .with_context(|| format!("Не удалось распарсить YAML: {}", path.display()))?;

        Ok(config)
    }

    /// Сохранить конфигурацию в файл
    pub fn save(&self, path: &Path) -> Result<()> {
        let content =
            serde_yaml::to_string(self).context("Не удалось сериализовать конфигурацию в YAML")?;

        fs::write(path, content)
            .with_context(|| format!("Не удалось записать файл: {}", path.display()))?;

        Ok(())
    }

    /// Инициализировать проект
    pub fn init(&mut self, project_name: String, path: &Path) -> Result<()> {
        self.project = project_name;

        // Установить окружение по умолчанию, если оно пустое
        if self.environment.is_empty() {
            self.environment = "dev".to_string();
        }

        self.save(path)
    }
}
