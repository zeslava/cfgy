use serde::{Deserialize, Serialize};

/// Конфигурация сервера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub crypto: CryptoConfig,
}

/// Конфигурация базы данных
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

/// Конфигурация сервера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Конфигурация криптографии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub master_key: String,
    pub jwt_secret: String,
}

impl Config {
    /// Загрузить конфигурацию из переменных окружения и файла
    pub fn load() -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?;

        builder.try_deserialize()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgres://postgres:postgres@localhost:5432/cfgy".to_string(),
                max_connections: 10,
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            crypto: CryptoConfig {
                master_key: std::env::var("CFGY_MASTER_KEY")
                    .unwrap_or_else(|_| "default-master-key-change-in-production".to_string()),
                jwt_secret: std::env::var("CFGY_JWT_SECRET")
                    .unwrap_or_else(|_| "default-jwt-secret-change-in-production".to_string()),
            },
        }
    }
}
