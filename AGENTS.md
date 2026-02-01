# Правила для AI агентов - CFGY Project

## Обзор проекта

**CFGY** — система управления конфигурацией и параметрами с полноценным стеком: **CLI + Server API + Web Dashboard**.

### Компоненты
- `server/` — Backend сервер (Axum framework, Rust)
- `cli/` — CLI клиент (Rust)
- `web/` — SPA фронтенд (React + TypeScript, Vite)
- `tests/` — Интеграционные smoke-тесты API

### Технологический стек
- **Backend:** Rust (edition 2024), Axum, PostgreSQL, sqlx
- **Frontend:** React 19.1.1, TypeScript 5.9.3, Tailwind CSS v4, Vite
- **База данных:** PostgreSQL
- **Шифрование:** AES-GCM для секретов, Argon2 для паролей

### Статус проекта
- ✅ **Этапы 0–3: MVP готов** (CLI, API, Web Dashboard)
- ⏳ **Этапы 4–6: в планах** (RBAC, команды, коллаборация, аудит, webhooks, production-ready)

## Архитектурные принципы

### Hexagonal Architecture (Server)

Сервер строго следует принципам Hexagonal Architecture (Ports & Adapters) для изоляции бизнес-логики от инфраструктуры.

#### Строгое разделение слоев

```
server/src/
├── application/    # Бизнес-логика (чистая, без зависимостей от инфраструктуры)
├── ports/          # Интерфейсы и доменные типы
│   ├── repositories.rs  # Трейты репозиториев
│   ├── crypto.rs       # Трейт CryptoService
│   └── entities.rs     # Доменные типы (User, Project, Configuration, ConfigKey)
└── adapters/       # Реализации
    ├── api/        # HTTP handlers (Axum)
    ├── persistence/ # PostgreSQL (sqlx)
    └── crypto/     # AES-GCM
```

#### Критические правила

1. **Application слой НИКОГДА не импортирует из adapters напрямую**
   - Бизнес-логика зависит только от трейтов в `ports/`
   - Никаких прямых зависимостей от БД, HTTP, криптографии

2. **Dependency Injection через трейты**
   - Все зависимости передаются через трейты из `ports/`
   - Конкретные реализации инжектируются в `main.rs`

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

- **Формат:** `000_name.up.sql` и `000_name.down.sql`
- **Нумерация:** `000_`, `001_`, `002_`, `003_` и т.д.
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
- Миграции применяются автоматически при запуске сервера в production
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
- Master key из переменной окружения `CFGY_MASTER_KEY`
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
- Статические файлы через `tower_http::fs::ServeDir`

### Аутентификация

- JWT для веб-сессий (session tokens)
- API tokens для CLI/CI (Bearer token в Authorization header)
- Cookies для веб-сессий (`cfgy_session`)
- ZID OAuth 2.0 (Authorization Code) через `/auth/zid/*` — вход через ZID IdP, обмен code на токены на бэкенде, сессия в cookie

### Ответы API

- Успешные ответы: `Json<T>` или `(StatusCode, Json<T>)`
- Ошибки: `(StatusCode, Json<ErrorResponse>)`
- Стандартные коды: 200 OK, 201 Created, 400 Bad Request, 401 Unauthorized, 404 Not Found

## Правила работы с CLI

### Структура команд

- Команды определены в `cli/src/commands.rs` через `clap`
- Основные команды: `init`, `use`, `set`, `get`, `list`, `delete`, `run`
- Подкоманды: `projects`, `token`

### Режимы работы

- **Server mode:** Если установлены `CFGY_API_TOKEN` и `CFGY_SERVER_URL`
- **Local mode:** Иначе, работа с `.cfgy.yaml`

### Конфигурация проекта

- Файл `.cfgy.yaml` в корне проекта
- Содержит: `project`, `configuration`
- Загрузка через `ProjectConfig::load()`

### Обработка ошибок

- Использовать `anyhow::Result` и `anyhow::bail!` для ошибок
- Понятные сообщения об ошибках на русском языке
- Валидация наличия проекта/configuration перед операциями

## Правила работы с фронтендом (React + TypeScript)

### Структура компонентов

- **UI компоненты:** `web/src/components/ui/` (переиспользуемые)
- **Domain компоненты:** `web/src/components/domain/` (специфичные для домена)
- **Pages:** `web/src/pages/` (одна страница = один файл)
- **Services:** `web/src/services/api.ts` (API клиент)

### Стилизация

- **Tailwind CSS v4** - все стили через utility классы
- **Design tokens:** определены в `web/tailwind.config.js`
- **Цвета:** использовать семантические имена (`text-text-dark`, `bg-basic-accent`)
- **Нет CSS Modules** или Styled Components
- **Глобальные стили:** только в `web/src/index.css`

### TypeScript

- Строгий режим (strict mode)
- Интерфейсы для всех props компонентов
- Типы экспортируются из компонентов
- Использовать `React.FC<Props>` для функциональных компонентов

### Паттерны компонентов

- Принимать `className` и spread `...rest` props
- Использовать `clsx` для условных классов
- Icons как React компоненты в `Icons.tsx`
- Context API для глобального состояния (`AuthContext`, `ToastContext`)

Пример:
```tsx
interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  icon?: React.ReactNode;
}

export const Button: React.FC<ButtonProps> = ({
  variant = "primary",
  size = "big-text",
  icon,
  children,
  className,
  ...rest
}) => {
  return (
    <button
      className={clsx(
        "inline-flex items-center justify-center gap-[10px]",
        getVariantClasses(variant),
        getSizeClasses(size),
        className
      )}
      {...rest}
    >
      {icon && <span>{icon}</span>}
      {children}
    </button>
  );
};
```

### Обработка ошибок API

- Автоматический logout при 401
- Показ ошибок через Toast
- Валидация форм на клиенте

## Правила работы с зависимостями

### Rust (Cargo)

- Workspace зависимости в корневом `Cargo.toml`
- Версии зависимостей через `workspace = true`
- Edition: 2024

### Node.js (npm)

- Зависимости в `web/package.json`
- React 19.1.1, TypeScript 5.9.3
- Vite для сборки

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

### Smoke-тесты

- Интеграционные тесты в `tests/smoke_tests.rs`
- Требуют запущенный сервер на `:8080`
- Запуск: `cargo test --bin smoke_tests`

### Unit-тесты

- Моки реализуют трейты из `ports/`
- Тестирование application слоя изолированно
- Тесты рядом с кодом в `#[cfg(test)]` модулях

## Правила работы с конфигурацией

### Сервер

- Конфигурация через переменные окружения
- Структура: `DATABASE_URL`, `SERVER_HOST`, `CFGY_MASTER_KEY` (без двойного подчеркивания)
- ZID OAuth 2.0: `ZID_ISSUER` (или `ZID_BASE_URL`), `ZID_CLIENT_ID`, `ZID_CLIENT_SECRET`, `ZID_REDIRECT_URI` (или `ZID_CALLBACK_URL`) — для входа через ZID по Authorization Code flow
- Значения по умолчанию в `config::Config::default()`
- Загрузка через `config::Config::load()` (читает переменные напрямую через `std::env::var()`)

### CLI

- Конфигурация проекта в `.cfgy.yaml`
- Переменные окружения: `CFGY_API_TOKEN`, `CFGY_SERVER_URL`
- Загрузка через `AppConfig::load()`

## Правила работы с Taskfile

### Команды

- Использовать `task` вместо `make`
- Основные команды: `task run`, `task web`, `task up`, `task down`, `task migrate`
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

- `.github/copilot-instructions.md` - подробные инструкции для Copilot
- `design.md` - архитектурные решения
- `plan.md` - план развития проекта
- `.cursor/rules/design_system_rules.mdc` - правила дизайн-системы для фронтенда
- `README.md` - общая документация проекта
