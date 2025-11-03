use anyhow::Result;

/// Конфигурация приложения, загружаемая из переменных окружения
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// API токен для аутентификации на сервере
    pub api_token: Option<String>,
    /// URL сервера CFGY
    pub server_url: String,
}

impl AppConfig {
    /// Загрузить конфигурацию из переменных окружения
    /// Вызывается один раз при старте приложения
    pub fn load() -> Self {
        Self {
            api_token: std::env::var("CFGY_API_TOKEN").ok(),
            server_url: std::env::var("CFGY_SERVER_URL")
                .unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
        }
    }

    /// Получить токен API, выдавая ошибку, если он не установлен
    pub fn require_api_token(&self) -> Result<String> {
        self.api_token.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "Для работы с сервером необходимо установить API токен:\n\
                export CFGY_API_TOKEN=<ваш_токен>\n\
                Токен можно создать в веб-дашборде."
            )
        })
    }

    /// Получить URL сервера, выдавая ошибку, если он не установлен
    pub fn require_server_url(&self) -> Result<String> {
        Ok(self.server_url.clone())
    }

    /// Проверить, настроен ли доступ к серверу (есть и токен, и URL)
    pub fn has_server_access(&self) -> bool {
        self.api_token.is_some() && !self.server_url.is_empty()
    }
}
