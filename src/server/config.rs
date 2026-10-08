use serde::{Deserialize, Serialize};

// Убрали зависимость от библиотеки config - читаем переменные окружения напрямую
// Это проще и понятнее, без необходимости в двойных подчеркиваниях

/// Конфигурация сервера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub crypto: CryptoConfig,
    pub oidc: OidcConfig,
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
    /// Разрешена ли регистрация по email/паролю (CFGY_ALLOW_REGISTRATION).
    /// None — только пока в системе нет ни одного пользователя (создание первого админа).
    pub allow_registration: Option<bool>,
}

/// Конфигурация OIDC-провайдера (OAuth 2.0 / OIDC для Web UI).
/// Работает с любым OIDC-совместимым провайдером: ZID, Keycloak, Auth0, Authentik и т.д.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    /// Публичный URL провайдера (issuer) — используется для редиректов в браузере.
    /// Должен быть доступен с клиента пользователя.
    /// Env: OIDC_ISSUER (fallback: ZID_ISSUER)
    pub issuer: String,

    /// URL провайдера для запросов с бэкенда (discovery, обмен code на токен, userinfo).
    /// Если задан — бэкенд ходит по нему; иначе используется issuer.
    /// Удобно, когда с сервера провайдер доступен по внутреннему адресу (например Tailscale).
    /// Env: OIDC_BASE_URL (fallback: ZID_BASE_URL)
    #[serde(default)]
    pub base_url: Option<String>,

    /// client_id приложения в OIDC-провайдере.
    /// Env: OIDC_CLIENT_ID (fallback: ZID_CLIENT_ID)
    pub client_id: String,

    /// client_secret для confidential client.
    /// Env: OIDC_CLIENT_SECRET (fallback: ZID_CLIENT_SECRET)
    pub client_secret: String,

    /// Полный callback URL (redirect_uri). Должен побайтово совпадать с redirect_uris в провайдере.
    /// Env: OIDC_REDIRECT_URI (fallback: ZID_REDIRECT_URI)
    pub redirect_uri: String,

    /// Не проверять TLS-сертификат при запросах к провайдеру (1/true/yes).
    /// Только для внутренних/домашних установок (self-signed). В production не использовать.
    /// Env: OIDC_TLS_INSECURE (fallback: ZID_TLS_INSECURE)
    #[serde(default)]
    pub tls_insecure: bool,

    /// Путь к PEM-файлу с доверенным CA (корневой или промежуточный сертификат).
    /// Предпочтительнее OIDC_TLS_INSECURE, т.к. проверка TLS остаётся включённой.
    /// Env: OIDC_CA_FILE (fallback: ZID_CA_FILE)
    #[serde(default)]
    pub ca_file: Option<String>,

    /// URL фронтенда для редиректа после успешного входа.
    /// Нужен только если OIDC_REDIRECT_URI ведёт на бэкенд (8080), а SPA на другом порту.
    /// Env: CFGY_FRONTEND_URL
    #[serde(default)]
    pub frontend_url: Option<String>,

    /// Идентификатор провайдера, сохраняемый в external_identities.provider.
    /// Менять только если нужно различать несколько провайдеров в одной БД.
    /// ⚠ Существующие пользователи ZID имеют provider='zid' в БД — задайте OIDC_PROVIDER_NAME=zid
    ///   если мигрируете с предыдущей версии и хотите сохранить привязку.
    /// Env: OIDC_PROVIDER_NAME (default: "oidc")
    pub provider: String,
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
        rng.fill(&mut buf)
            .expect("Не удалось сгенерировать случайные байты");
        hex::encode(&buf)
    }

    /// Загрузить конфигурацию из переменных окружения
    pub fn load() -> Result<Self, String> {
        // Читать OIDC_* с fallback на ZID_* для обратной совместимости
        let oidc_or_zid = |oidc: &str, zid: &str| -> Option<String> {
            std::env::var(oidc)
                .ok()
                .filter(|s| !s.is_empty())
                .or_else(|| std::env::var(zid).ok().filter(|s| !s.is_empty()))
        };
        // Для bool: если OIDC_* задан (любое значение), берём его; иначе — fallback на ZID_*.
        // Иначе явный OIDC_*=false не сможет перекрыть старый ZID_*=true.
        let oidc_or_zid_bool = |oidc: &str, zid: &str| -> bool {
            match std::env::var(oidc) {
                Ok(v) if !v.is_empty() => Self::parse_bool_env(oidc),
                _ => Self::parse_bool_env(zid),
            }
        };
        // Автогенерация мастер-ключа недопустима: после перезапуска данные не расшифруются
        let master_key = match std::env::var("CFGY_MASTER_KEY") {
            Ok(k) if !k.is_empty() => k,
            _ => {
                return Err(format!(
                    "CFGY_MASTER_KEY не задан. Сгенерируйте: cfgy generate-secrets или openssl rand -hex 32\n    Например: CFGY_MASTER_KEY={}",
                    Self::generate_random_hex(32)
                ));
            }
        };

        let jwt_secret = match std::env::var("CFGY_JWT_SECRET") {
            Ok(s) if !s.is_empty() => s,
            _ => {
                let secret = Self::generate_random_hex(32);
                eprintln!(
                    "⚠️  CFGY_JWT_SECRET не задан — сгенерирован автоматически (сессии сбросятся после перезапуска)"
                );
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
                allow_registration: match std::env::var("CFGY_ALLOW_REGISTRATION") {
                    Ok(v) if !v.is_empty() => Some(Self::parse_bool_env("CFGY_ALLOW_REGISTRATION")),
                    _ => None,
                },
            },
            crypto: CryptoConfig {
                master_key,
                jwt_secret,
            },
            oidc: OidcConfig {
                issuer: oidc_or_zid("OIDC_ISSUER", "ZID_ISSUER")
                    .unwrap_or_else(|| "http://localhost:5555".to_string()),
                base_url: oidc_or_zid("OIDC_BASE_URL", "ZID_BASE_URL"),
                client_id: oidc_or_zid("OIDC_CLIENT_ID", "ZID_CLIENT_ID").unwrap_or_default(),
                client_secret: oidc_or_zid("OIDC_CLIENT_SECRET", "ZID_CLIENT_SECRET")
                    .unwrap_or_default(),
                redirect_uri: oidc_or_zid("OIDC_REDIRECT_URI", "ZID_REDIRECT_URI")
                    .unwrap_or_else(|| "http://localhost:5173/auth/oidc/callback".to_string()),
                tls_insecure: oidc_or_zid_bool("OIDC_TLS_INSECURE", "ZID_TLS_INSECURE"),
                ca_file: oidc_or_zid("OIDC_CA_FILE", "ZID_CA_FILE"),
                frontend_url: std::env::var("CFGY_FRONTEND_URL")
                    .ok()
                    .filter(|s| !s.is_empty()),
                provider: std::env::var("OIDC_PROVIDER_NAME")
                    .ok()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "oidc".to_string()),
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

        // Значения-заглушки из .env.example и старого docker-compose.yml — публично известны
        const PLACEHOLDERS: &[&str] = &[
            "your-master-key-minimum-32-characters-long",
            "your-jwt-secret-minimum-16-chars",
            "default-master-key-change-in-production",
            "default-jwt-secret-change-in-production",
        ];

        if self.crypto.master_key.len() < MIN_MASTER_KEY_LEN {
            return Err(format!(
                "CFGY_MASTER_KEY слишком короткий ({} символов, минимум {})",
                self.crypto.master_key.len(),
                MIN_MASTER_KEY_LEN
            ));
        }

        if self.crypto.jwt_secret.len() < MIN_JWT_SECRET_LEN {
            return Err(format!(
                "CFGY_JWT_SECRET слишком короткий ({} символов, минимум {})",
                self.crypto.jwt_secret.len(),
                MIN_JWT_SECRET_LEN
            ));
        }

        if PLACEHOLDERS.contains(&self.crypto.master_key.as_str()) {
            return Err(
                "CFGY_MASTER_KEY содержит значение-заглушку из примера, задайте свой ключ".into(),
            );
        }

        if PLACEHOLDERS.contains(&self.crypto.jwt_secret.as_str()) {
            return Err(
                "CFGY_JWT_SECRET содержит значение-заглушку из примера, задайте свой секрет".into(),
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
                allow_registration: Some(false),
            },
            crypto: CryptoConfig {
                master_key: Self::generate_random_hex(32),
                jwt_secret: Self::generate_random_hex(32),
            },
            oidc: OidcConfig {
                issuer: "http://localhost:5555".to_string(),
                base_url: None,
                client_id: String::new(),
                client_secret: String::new(),
                redirect_uri: "http://localhost:5173/auth/oidc/callback".to_string(),
                tls_insecure: false,
                ca_file: None,
                frontend_url: None,
                provider: "oidc".to_string(),
            },
        }
    }
}
