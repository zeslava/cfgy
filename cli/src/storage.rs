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
    /// Текущая configuration проекта (например: dev/staging/prod)
    ///
    /// В cfgy.yaml это поле хранится как `configuration`.
    #[serde(default = "default_configuration")]
    pub configuration: String,
}

fn default_project_name() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "test-project-name".to_string())
}

fn default_configuration() -> String {
    "dev".to_string()
}

impl ProjectConfig {
    /// Найти путь к конфигурационному файлу
    pub fn find_config_path() -> Result<PathBuf> {
        let mut dir =
            std::env::current_dir().context("Не удалось определить текущую директорию")?;

        loop {
            let candidate = dir.join("cfgy.yaml");
            if candidate.exists() {
                return Ok(candidate);
            }

            if !dir.pop() {
                break;
            }
        }

        Err(anyhow::anyhow!(
            "Конфигурационный файл не найден (ожидался cfgy.yaml в текущей директории или выше)"
        ))
    }

    /// Путь к конфигурационному файлу в текущей директории
    pub fn config_path_in_current_dir() -> Result<PathBuf> {
        std::env::current_dir()
            .map(|p| p.join("cfgy.yaml"))
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
                configuration: default_configuration(),
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
    pub fn init(
        &mut self,
        project_name: String,
        configuration: Option<String>,
        path: &Path,
    ) -> Result<()> {
        self.project = project_name;

        if let Some(c) = configuration {
            self.configuration = c;
        } else if self.configuration.is_empty() {
            self.configuration = "dev".to_string();
        }

        self.save(path)
    }
}
