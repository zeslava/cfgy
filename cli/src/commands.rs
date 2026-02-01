use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cfgy")]
#[command(about = "CFGY - Configuration & Secrets Management")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Инициализация проекта CFGY
    Init {
        /// Имя проекта
        #[arg(short, long)]
        project: Option<String>,
    },
    /// Использовать configuration по умолчанию для текущего проекта (записывается в .cfgy.yaml)
    Use {
        /// Имя configuration (например: dev, staging, prod)
        configuration: String,
    },
    /// Установить значение конфига
    Set {
        /// Ключ конфигурации
        key: String,
        /// Значение конфигурации
        value: String,
        /// Configuration (если не указана — берётся из .cfgy.yaml)
        #[arg(short, long)]
        configuration: Option<String>,
    },
    /// Получить значение конфига
    Get {
        /// Ключ конфигурации
        key: String,
        /// Configuration (если не указана — берётся из .cfgy.yaml)
        #[arg(short, long)]
        configuration: Option<String>,
        /// Формат вывода (json, yaml, env)
        #[arg(short, long, default_value = "env")]
        format: String,
    },
    /// Список всех конфигов
    List {
        /// Configuration (если не указана — берётся из .cfgy.yaml)
        #[arg(short, long)]
        configuration: Option<String>,
        /// Формат вывода (json, yaml, env)
        #[arg(short, long, default_value = "env")]
        format: String,
    },
    /// Удалить конфиг
    Delete {
        /// Ключ конфигурации
        key: String,
        /// Configuration (если не указана — берётся из .cfgy.yaml)
        #[arg(short, long)]
        configuration: Option<String>,
    },
    /// Запустить команду с переменными окружения из конфигов
    Run {
        /// Команда для выполнения
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
    Projects {
        /// Действие
        #[command(subcommand)]
        action: ProjectsAction,
    },
    /// Работа с API токенами
    Token {
        /// Действие
        #[command(subcommand)]
        action: TokenAction,
    },
    /// Генерация секретных ключей для сервера
    GenerateSecrets {
        /// Формат вывода (env, shell)
        #[arg(short, long, default_value = "env")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum ProjectsAction {
    /// Список проектов
    List,
    /// Создать проект
    Create,
    /// Удалить проект
    Delete,
    /// Переименовать проект
    Rename,
    /// Просмотр проекта
    View,
    /// Редактировать проект
    Edit,
}

#[derive(Subcommand)]
pub enum TokenAction {
    /// Показать установленный токен
    List,
}
