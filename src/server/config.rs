use serde::{Deserialize, Serialize};

// Убрали зависимость от библиотеки config - читаем переменные окружения напрямую
// Это проще и понятнее, без необходимости в двойных подчеркиваниях

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

/// Конфигурация интеграции с ZID (OAuth 2.0 / OIDC для Web UI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZidConfig {
    /// Публичный URL ZID (issuer) — используется для редиректов в браузере (страница входа ZID).
    /// Должен быть доступен с клиента пользователя.
    pub issuer: String,

    /// URL ZID для запросов с бэкенда (discovery, обмен code на токен, userinfo).
    /// Если задан ZID_BASE_URL — бэкенд ходит по нему; иначе используется issuer.
    /// Удобно, когда с сервера ZID доступен по внутреннему адресу (например Tailscale).
    #[serde(default)]
    pub base_url: Option<String>,

    /// client_id, выданный для этого приложения в ZID (oidc_clients и т.п.)
    pub client_id: String,

    /// client_secret для confidential client
    pub client_secret: String,

    /// Полный callback URL (redirect_uri). Должен побайтово совпадать с redirect_uris в ZID.
    /// Например: https://our-app.com/auth/zid/callback или http://localhost:5173/auth/zid/callback
    pub redirect_uri: String,

    /// Не проверять TLS-сертификат при запросах к ZID (1/true/yes).
    /// Только для внутренних/домашних установок (Tailscale, self-signed). В production не использовать.
    #[serde(default)]
    pub tls_insecure: bool,

    /// Путь к PEM-файлу с доверенным CA (корневой или промежуточный сертификат).
    /// Для ZID с сертификатом от Tailscale: положите корневой CA Tailscale в файл и укажите путь (например /etc/ssl/tailscale-ca.pem).
    /// Предпочтительнее ZID_TLS_INSECURE, т.к. проверка TLS остаётся включённой.
    #[serde(default)]
    pub ca_file: Option<String>,
}

/// Конфигурация криптографии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub master_key: String,
    pub jwt_secret: String,
}

impl Config {
    /// Сгенерировать случайный hex-ключ заданной длины (в байтах).
    fn generate_random_hex(len: usize) -> String {
        use ring::rand::{SecureRandom, SystemRandom};
        let rng = SystemRandom::new();
        let mut buf = vec![0u8; len];
        rng.fill(&mut buf).expect("Не удалось сгенерировать случайные байты");
        hex::encode(&buf)
    }

    /// Загрузить конфигурацию из переменных окружения
    pub fn load() -> Result<Self, String> {
        let master_key = match std::env::var("CFGY_MASTER_KEY") {
            Ok(k) if !k.is_empty() => k,
            _ => {
                let key = Self::generate_random_hex(32);
                eprintln!("⚠️  CFGY_MASTER_KEY не задан — сгенерирован автоматически (данные не будут расшифрованы после перезапуска!)");
                eprintln!("    Сохраните в .env или переменные окружения:");
                eprintln!("    CFGY_MASTER_KEY={}", key);
                key
            }
        };

        let jwt_secret = match std::env::var("CFGY_JWT_SECRET") {
            Ok(s) if !s.is_empty() => s,
            _ => {
                let secret = Self::generate_random_hex(32);
                eprintln!("⚠️  CFGY_JWT_SECRET не задан — сгенерирован автоматически (сессии сбросятся после перезапуска)");
                eprintln!("    Сохраните в .env или переменные окружения:");
                eprintln!("    CFGY_JWT_SECRET={}", secret);
                secret
            }
        };

        let config = Config {
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://postgres:postgres@localhost:5432/cfgy".to_string()
                }),
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(20),
            },
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("SERVER_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(8080),
            },
            crypto: CryptoConfig {
                master_key,
                jwt_secret,
            },
            zid: ZidConfig {
                issuer: std::env::var("ZID_ISSUER")
                    .unwrap_or_else(|_| "http://localhost:5555".to_string()),
                base_url: std::env::var("ZID_BASE_URL").ok().filter(|s| !s.is_empty()),
                client_id: std::env::var("ZID_CLIENT_ID").unwrap_or_default(),
                client_secret: std::env::var("ZID_CLIENT_SECRET").unwrap_or_default(),
                redirect_uri: std::env::var("ZID_REDIRECT_URI")
                    .unwrap_or_else(|_| "http://localhost:5173/auth/zid/callback".to_string()),
                tls_insecure: Self::parse_bool_env("ZID_TLS_INSECURE"),
                ca_file: std::env::var("ZID_CA_FILE").ok().filter(|s| !s.is_empty()),
            },
        };

        // Валидация конфигурации
        config.validate().map_err(|e| e.to_string())?;

        Ok(config)
    }

    /// Валидация конфигурации
    pub fn validate(&self) -> Result<(), String> {
        const MIN_MASTER_KEY_LEN: usize = 32;
        const MIN_JWT_SECRET_LEN: usize = 16;

        if self.crypto.master_key.len() < MIN_MASTER_KEY_LEN {
            eprintln!(
                "⚠️  CFGY_MASTER_KEY слишком короткий ({} символов, минимум {})",
                self.crypto.master_key.len(),
                MIN_MASTER_KEY_LEN
            );
        }

        if self.crypto.jwt_secret.len() < MIN_JWT_SECRET_LEN {
            eprintln!(
                "⚠️  CFGY_JWT_SECRET слишком короткий ({} символов, минимум {})",
                self.crypto.jwt_secret.len(),
                MIN_JWT_SECRET_LEN
            );
        }

        Ok(())
    }

    /// Читает переменную окружения как булево (1, true, yes = true).
    fn parse_bool_env(name: &str) -> bool {
        let v = match std::env::var(name) {
            Ok(s) => s.to_lowercase(),
            Err(_) => return false,
        };
        matches!(v.as_str(), "1" | "true" | "yes")
    }
}

impl Default for Config {
    fn default() -> Self {
        // Default вызывается только как fallback при ошибке load(),
        // поэтому тоже генерируем ключи автоматически
        Self {
            database: DatabaseConfig {
                url: "postgres://postgres:postgres@localhost:5432/cfgy".to_string(),
                max_connections: 20,
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            crypto: CryptoConfig {
                master_key: Self::generate_random_hex(32),
                jwt_secret: Self::generate_random_hex(32),
            },
            zid: ZidConfig {
                issuer: "http://localhost:5555".to_string(),
                base_url: None,
                client_id: String::new(),
                client_secret: String::new(),
                redirect_uri: "http://localhost:5173/auth/zid/callback".to_string(),
                tls_insecure: false,
                ca_file: None,
            },
        }
    }
}
