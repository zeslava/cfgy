# Правила для AI агентов - CFGY Project

## Обзор проекта

**CFGY** — система управления конфигурацией и параметрами с полноценным стеком: **CLI + Server API + Web Dashboard**.

### Компоненты
Один Rust-бинарь `cfgy` (`src/main.rs`) — CLI и сервер (`cfgy serve`):
- `src/server/` — Backend сервер (Axum)
- `src/cli/` — CLI клиент
- `web/` — SPA фронтенд (Svelte 5 + SvelteKit), сборка `web/build/` встраивается в бинарь (rust-embed)
- `tests/e2e.rs` — E2E-тесты против живого сервера
- `migrations/` — SQL-миграции

### Технологический стек
- **Backend:** Rust (edition 2024), Axum, PostgreSQL, sqlx
- **Frontend:** Svelte 5, SvelteKit, TypeScript, Vite (под капотом SvelteKit)
- **База данных:** PostgreSQL 18
- **Шифрование:** AES-GCM для секретов, Argon2 для паролей

### Статус проекта
- MVP готов (CLI, API, Web Dashboard)

## Архитектурные принципы

### Hexagonal Architecture (Server)

Сервер строго следует принципам Hexagonal Architecture (Ports & Adapters) для изоляции бизнес-логики от инфраструктуры.

#### Строгое разделение слоев

```
src/server/
├── application/    # Бизнес-логика (чистая, без зависимостей от инфраструктуры)
├── ports/          # Интерфейсы и доменные типы
│   ├── repositories.rs  # Трейты репозиториев
│   ├── crypto.rs       # Трейт CryptoService
│   └── entities.rs     # Доменные типы (User, Project, Configuration, ConfigKey)
└── adapters/       # Реализации
    ├── api/        # HTTP handlers, router, auth (JWT/Bearer/OIDC), static_assets
    ├── persistence/ # PostgreSQL (sqlx)
    └── crypto/     # AES-GCM
```

#### Критические правила

1. **Application слой НИКОГДА не импортирует из adapters напрямую**
   - Бизнес-логика зависит только от трейтов в `ports/`
   - Никаких прямых зависимостей от БД, HTTP, криптографии

2. **Dependency Injection через трейты**
   - Все зависимости передаются через трейты из `ports/` (generic-параметры сервисов)
   - Конкретные реализации инжектируются при запуске сервера (`src/server/mod.rs`)

3. **Тестирование**
   - Моки реализуют трейты из `ports/`
   - Application слой тестируется изолированно

## Правила работы с Rust кодом

### Именование

- **Структуры:** PascalCase (`ProjectService`, `UserRepository`)
- **Трейты:** PascalCase с суффиксом, описывающим роль (`UserRepository`, `CryptoService`)
- **Функции:** snake_case (`create_project`, `find_by_email`)
- **Константы:** SCREAMING_SNAKE_CASE (`CFGY_SESSION_COOKIE_NAME`)
- **Модули:** snake_case (`application`, `adapters`)

### Комментарии и документация

- **Все комментарии на русском языке** (согласно правилам пользователя)
- Документация для публичных API через `///`
- Комментарии объясняют "почему", а не "что"

Пример:
```rust
/// Создать новый проект
/// 
/// Валидирует имя проекта и создает запись в БД через репозиторий.
pub async fn create_project(
    &self,
    name: &str,
    description: Option<&str>,
    owner_id: Uuid,
) -> Result<Project> {
    // Валидация имени проекта
    if name.is_empty() {
        anyhow::bail!("Имя проекта не может быть пустым");
    }
    // ...
}
```

### Обработка ошибок

- Использовать `anyhow::Result<T>` для общего error handling
- Использовать `anyhow::Context` для добавления контекста к ошибкам
- Валидация входных данных в application слое
- Ошибки БД оборачиваются с контекстом через `.context()`

Пример:
```rust
let project = self.project_repo
    .find_by_id(id)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Проект не найден"))
    .context("Не удалось получить проект из БД")?;
```

### Типы и структуры

- Доменные типы в `ports/entities.rs` с `#[derive(FromRow)]` для sqlx
- Request/Response типы в handlers (не в ports)
- Использовать `Uuid` для идентификаторов
- Использовать `chrono::DateTime<Utc>` для временных меток

Пример:
```rust
// ports/entities.rs
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Async/Await

- Все функции репозиториев и сервисов async
- Использовать `async_trait` для трейтов с async методами
- Пул подключений через `PgPoolOptions`

## Правила работы с базой данных

### Миграции

#### Формат файлов миграций

- **Каталог:** `migrations/`
- **Формат:** `NNN_name.up.sql` и `NNN_name.down.sql`
- **Нумерация:** `001_`, `002_`, … (следующий свободный номер после последней миграции)
- **Имена:** описательные, snake_case (например: `001_initial_schema.up.sql`, `002_rename_environments_to_configurations.up.sql`)
- **Up миграция:** содержит изменения для применения миграции
- **Down миграция:** содержит обратные изменения для отката миграции

#### Создание миграций

- Всегда создавать пару файлов: `.up.sql` и `.down.sql`
- Всегда проверять существование: `IF NOT EXISTS` в up миграциях
- Down миграции должны полностью откатывать изменения up миграции
- Обратная совместимость при изменении схемы

#### Применение миграций

- Через `task migrate` (использует `sqlx migrate run`)
- Переменная `DATABASE_URL` должна быть установлена
- Миграции применяются автоматически при запуске сервера
- Откат через `sqlx migrate revert` (применяет down миграции)

#### Пример структуры миграции

**001_initial_schema.up.sql:**
```sql
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    -- ...
);
```

**001_initial_schema.down.sql:**
```sql
DROP TABLE IF EXISTS users CASCADE;
-- ...
```

### Репозитории

- Реализация трейтов из `ports/repositories.rs`
- Использовать `sqlx::query_as` для типизированных запросов
- Всегда использовать параметризованные запросы (без конкатенации строк)
- Обработка `Option<T>` для `find_by_*` методов

Пример:
```rust
async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
    .context("Ошибка при поиске пользователя")?;
    
    Ok(user)
}
```

### Шифрование

- Параметры хранятся как `BYTEA` (encrypted_value)
- Шифрование через `CryptoService` trait
- Master key из переменной окружения `CFGY_MASTER_KEY` (обязателен: без него сервер не стартует)
- Пароли хэшируются через Argon2 (не шифруются)

## Правила работы с API (Axum)

### Handlers

- Handlers в `adapters/api/handlers.rs`
- Использовать `State<AppState>` для доступа к сервисам
- Извлечение пользователя через `extract_user()` из headers
- Возвращать `Result<Json<T>, (StatusCode, Json<ErrorResponse>)>` для ошибок

Пример:
```rust
pub async fn create_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), (StatusCode, Json<ErrorResponse>)> {
    let user = extract_user(&headers, &state).await?;
    
    let project = state
        .project_service
        .create_project(&request.name, request.description.as_deref(), user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e.to_string() }),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(ProjectResponse { /* ... */ }),
    ))
}
```

### Роутинг

- Роуты определяются в `adapters/api/router.rs`
- API endpoints под префиксом `/api/`
- Auth endpoints: `/api/auth/*`
- Статика встроена в бинарь (`static_assets.rs`, rust-embed); `CFGY_WEB_ROOT` — читать с диска; SPA fallback на `index.html`

### Аутентификация

- JWT для веб-сессий (session tokens)
- API tokens для CLI/CI (Bearer token в Authorization header)
- Cookies для веб-сессий (`cfgy_session`)
- OIDC SSO (Authorization Code) через `/auth/oidc/*` (алиасы `/oauth/oidc/*`, `/auth/zid/*`, `/oauth/zid/*`) — обмен code на токены на бэкенде, сессия в cookie
- Пользователь OIDC привязывается только по паре (provider, sub), не по email/username
- Регистрация email/паролем: только первый пользователь, если не задан `CFGY_ALLOW_REGISTRATION`

### Ответы API

- Успешные ответы: `Json<T>` или `(StatusCode, Json<T>)`
- Ошибки: `(StatusCode, Json<ErrorResponse>)`
- Стандартные коды: 200 OK, 201 Created, 400 Bad Request, 401 Unauthorized, 404 Not Found

## Правила работы с CLI

### Структура команд

- Команды определены в `src/cli/commands.rs` через `clap`, обработка — в `src/main.rs`
- Основные команды: `init`, `use`, `set`, `get`, `list`, `env`, `delete`, `run`
- Подкоманды: `projects`, `token`, `config`, `completions`
- Служебные: `setup`, `generate-secrets`, `serve`
- Форматы вывода (env/json/yaml) — `src/cli/formats.rs`

### Режимы работы

- **Server mode:** Если установлены `CFGY_TOKEN` и `CFGY_SERVER_URL`
- **Local mode:** Иначе, работа с `cfgy.yaml`

### Конфигурация проекта

- Файл `cfgy.yaml` в корне проекта
- Содержит: `project`, `configuration`
- Загрузка через `ProjectConfig::load()`

### Обработка ошибок

- Использовать `anyhow::Result` и `anyhow::bail!` для ошибок
- Понятные сообщения об ошибках на русском языке
- Валидация наличия проекта/configuration перед операциями

## Правила работы с фронтендом (Svelte 5 + SvelteKit)

### Структура

- **Маршруты и страницы:** `web/src/routes/` (файловая маршрутизация SvelteKit: `+page.svelte`, `+layout.svelte`)
- **Компоненты:** `web/src/lib/components/` (переиспользуемые: Button, Input, Modal, Header и т.д.)
- **Логика и API:** `web/src/lib/api.ts`, `web/src/lib/auth.ts`, `web/src/lib/toast.ts`
- **Статика:** `web/static/` (favicon, logo, robots.txt) — копируется в корень сборки
- **Сборка:** `npm run build` → артефакты в `web/build/` (adapter-static)

### Стилизация

- Стили в компонентах или в `web/src/app.css` (глобальные)
- Использовать семантические имена для цветов и отступов

### TypeScript

- Строгий режим (strict mode)
- Типы для props и API-ответов
- Типы экспортируются из модулей при необходимости

### Паттерны компонентов (Svelte)

- Svelte 5 runes: props через `let { ... }: Props = $props()`
- События через callback-props (`onClose`, `onClick`)
- Глобальное состояние: stores (`auth`, toast) в `web/src/lib/`

### Обработка ошибок API

- Автоматический logout при 401
- Показ ошибок через Toast
- Валидация форм на клиенте

## Правила работы с зависимостями

### Rust (Cargo)

- Один крейт `cfgy`, зависимости в корневом `Cargo.toml`
- Edition: 2024

### Node.js (npm)

- Зависимости в `web/package.json`
- Svelte 5, SvelteKit, TypeScript
- Vite для сборки (через SvelteKit)

## Правила безопасности

### Шифрование

- Все секреты шифруются перед сохранением в БД
- Master key из переменной окружения (НЕ в коде)
- AES-GCM для шифрования значений
- Argon2 для хэширования паролей

### Аутентификация

- API tokens создаются только через веб-дашборд
- JWT для веб-сессий с истечением срока
- Валидация токенов на каждом запросе

### Валидация

- Валидация входных данных в application слое
- Проверка прав доступа перед операциями
- Санитизация пользовательского ввода

## Правила тестирования

### E2E-тесты

- Тесты в `tests/e2e.rs`
- Запуск: `task e2e` (`scripts/run-e2e.sh`: сборка, миграции, запуск сервера на `:8080`, тесты, остановка)
- Требуют доступную БД (`task up`); пишут в БД из `DATABASE_URL`

### Unit-тесты

- Моки реализуют трейты из `ports/`
- Тестирование application слоя изолированно
- Тесты рядом с кодом в `#[cfg(test)]` модулях

## Правила работы с конфигурацией

### Сервер

- Конфигурация через переменные окружения
- Переменные: `DATABASE_URL`, `DATABASE_MAX_CONNECTIONS`, `SERVER_HOST`, `SERVER_PORT`, `CFGY_MASTER_KEY` (обязательна), `CFGY_JWT_SECRET`, `CFGY_ALLOW_REGISTRATION`, `CFGY_FRONTEND_URL`, `CFGY_WEB_ROOT`
- OIDC SSO: `OIDC_ISSUER` (или `OIDC_BASE_URL`), `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_REDIRECT_URI`, `OIDC_PROVIDER_NAME`, `OIDC_TLS_INSECURE`, `OIDC_CA_FILE`; `ZID_*` принимаются как fallback
- Значения по умолчанию в `config::Config::default()`
- Загрузка через `config::Config::load()` (читает переменные напрямую через `std::env::var()`)

### CLI

- Конфигурация проекта в `cfgy.yaml`
- Подключение к серверу: переменные окружения `CFGY_TOKEN`, `CFGY_SERVER_URL` или центральный конфиг `~/.cfgy/config.yaml` (env переопределяет файл)
- Загрузка через `AppConfig::load()` (сначала читается ~/.cfgy/config.yaml, затем переопределение из env)

## Правила работы с Taskfile

### Команды

- Использовать `task` вместо `make`
- Основные команды: `task run`, `task web`, `task up`, `task down`, `task migrate`, `task e2e`, `task generate-secrets`
- Справка: `task` или `task --list`

## Общие правила для AI агентов

### При добавлении нового функционала

1. Определить, в каком слое должна быть логика (ports/application/adapters)
2. Если нужна новая таблица - создать миграцию (пару `.up.sql` и `.down.sql`)
3. Добавить трейт в `ports/` если нужен новый репозиторий/сервис
4. Реализовать в `application/` (бизнес-логика)
5. Реализовать в `adapters/` (конкретная реализация)
6. Добавить handler в `adapters/api/handlers.rs` если нужен API endpoint
7. Добавить route в `adapters/api/router.rs`

### При изменении существующего кода

- Сохранять архитектурные принципы (Hexagonal Architecture)
- Не нарушать границы слоев
- Обновлять тесты при изменении логики
- Обновлять документацию при изменении API

### При работе с ошибками

- Всегда добавлять контекст через `.context()`
- Использовать понятные сообщения на русском языке
- Валидировать входные данные
- Возвращать правильные HTTP статус коды

### При работе с безопасностью

- Никогда не логировать секреты
- Всегда шифровать перед сохранением в БД
- Валидировать права доступа
- Использовать параметризованные запросы

### При работе с миграциями

- Всегда создавать пару файлов: `.up.sql` и `.down.sql`
- Down миграция должна полностью откатывать изменения up миграции
- Использовать `IF NOT EXISTS` в up миграциях
- Тестировать откат миграций перед коммитом

## Полезные ссылки

- `.github/copilot-instructions.md` - инструкции для Copilot
- `design.md` - исходный план проекта (исторический документ)
- `README.md` - общая документация проекта
