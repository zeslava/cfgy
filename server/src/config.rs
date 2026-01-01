use serde::{Deserialize, Serialize};

/// Конфигурация сервера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub crypto: CryptoConfig,
    pub zid: ZidConfig,
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

/// Конфигурация интеграции с ZID (SSO для Web UI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZidConfig {
    /// Base URL сервиса ZID, например: http://localhost:5555
    pub base_url: String,

    /// Полный callback URL, который ZID будет дергать после логина (service/return_to).
    /// Для dev через web: http://localhost:5173/auth/zid/callback
    pub callback_url: String,
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
        let mut builder =
            config::Config::builder().add_source(config::Environment::default().separator("__"));

        // Добавляем значения по умолчанию для crypto, если они не заданы
        let default = Config::default();
        builder = builder
            .set_default("crypto.master_key", default.crypto.master_key.clone())?
            .set_default("crypto.jwt_secret", default.crypto.jwt_secret.clone())?;

        let config = builder.build()?;
        config.try_deserialize()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgres://postgres:postgres@localhost:5432/cfgy".to_string(),
                // Увеличено значение по умолчанию для предотвращения таймаутов
                max_connections: 20,
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
            zid: ZidConfig {
                base_url: std::env::var("ZID__BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:5555".to_string()),
                callback_url: std::env::var("ZID__CALLBACK_URL")
                    .unwrap_or_else(|_| "http://localhost:5173/auth/zid/callback".to_string()),
            },
        }
    }
}
