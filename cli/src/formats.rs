use anyhow::Result;
use std::collections::HashMap;

/// Экранирует значение для формата .env (подходит для docker --env-file).
fn escape_env_value(v: &str) -> String {
    let needs_quotes = v.contains('\n')
        || v.contains('"')
        || v.contains('\\')
        || v.contains('#')
        || v.starts_with(' ')
        || v.ends_with(' ')
        || v.starts_with(';'); // чтобы не трактовалось как комментарий в shell/.env
    if !needs_quotes {
        return v.to_string();
    }
    let escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

/// Формат вывода конфигов
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Json,
    Yaml,
    Env,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "yaml" | "yml" => Ok(Self::Yaml),
            "env" | "dotenv" => Ok(Self::Env),
            _ => anyhow::bail!("Неизвестный формат: {}. Поддерживаются: json, yaml, env", s),
        }
    }

    /// Вывести конфиги в указанном формате
    pub fn output(&self, configs: &HashMap<String, String>) -> Result<String> {
        match self {
            Self::Json => serde_json::to_string_pretty(configs)
                .map_err(|e| anyhow::anyhow!("Ошибка сериализации JSON: {}", e)),
            Self::Yaml => serde_yaml::to_string(configs)
                .map_err(|e| anyhow::anyhow!("Ошибка сериализации YAML: {}", e)),
            Self::Env => Ok(configs
                .iter()
                .map(|(k, v)| format!("{}={}", k, escape_env_value(v)))
                .collect::<Vec<_>>()
                .join("\n")),
        }
    }
}
