# CFGY

Система управления конфигурацией и параметрами с полноценным стеком: **CLI + Server API + Web Dashboard**.

> Всё описание проекта и инструкции по запуску — в этом корневом `README.md`.

## Использование

CLI работает в контексте проекта (файл `cfgy.yaml`: имя проекта и текущая configuration). **Configuration** — это окружение (dev, prod и т.д.); **параметры** — пары ключ–значение внутри выбранной configuration.

Типичный порядок: запустить сервер (`task run`) → открыть `http://localhost:8080` → зарегистрироваться/войти → создать API-токен → экспортировать `CFGY_SERVER_URL` и `CFGY_TOKEN` → в проекте выполнить `cfgy init` и `cfgy use dev` → использовать `cfgy get/set/list/run`.

Типичные команды:
```bash
cfgy use dev                    # выбрать configuration по умолчанию (записывается в cfgy.yaml)
cfgy set DATABASE_URL postgres://...
cfgy get DATABASE_URL
cfgy list                       # список параметров текущей configuration
cfgy run -- <command> [args...] # запуск команды с env из текущей configuration

cfgy generate-secrets           # сгенерировать CFGY_MASTER_KEY и CFGY_JWT_SECRET
cfgy generate-secrets --format shell
```

### Подключение CLI к серверу
Контекст проекта — `cfgy.yaml` (имя проекта и текущая configuration). Подключение к серверу задаётся переменными окружения или центральным конфигом `~/.cfgy/config.yaml` (переменные окружения имеют приоритет над файлом).

**Переменные окружения:**
- `CFGY_SERVER_URL` — URL сервера (например `http://localhost:8080` или `https://cfgy.example.com`). Если схема не указана, используется `http://`.
- `CFGY_TOKEN` — API-токен (создаётся в веб-дашборде)

**Центральный конфиг** `~/.cfgy/config.yaml` (YAML):

```yaml
server_url: "http://localhost:8080"
token: "your-token"
```

Рекомендуется права доступа на файл `0600`. Токен и URL можно задать либо в env, либо в этом файле; env переопределяет файл (удобно для CI).

```bash
export CFGY_SERVER_URL=http://localhost:8080
export CFGY_TOKEN="<token-from-dashboard>"
```

### Токены и аутентификация

- **JWT (session token)** — веб-сессия (логин в дашборде)
- **API-токен** — доступ из CLI/CI (создаётся только в веб-дашборде)

## Установка

### Требования

- **Docker + Docker Compose** — для PostgreSQL (`task up`) или полного стека в контейнерах
- **Rust** (edition 2024) — для сервера и CLI
- **Node.js + npm** — для сборки и dev-режима веб-приложения (`web/build`)
- **Task** — [taskfile.dev](https://taskfile.dev), установка: `go install github.com/go-task/task/v3/cmd/task@latest` или пакетный менеджер
- **sqlx-cli** (для миграций без Task): `cargo install sqlx-cli --no-default-features --features postgres`

### Быстрый старт (Docker + Taskfile)

```bash
task                    # список команд

task up                 # поднять PostgreSQL в Docker (порт 5433)
task run                # собрать сервер (release), применить миграции, запустить сервер локально
task web                # в отдельном терминале: веб dev-сервер на http://localhost:5173

task down               # остановить контейнеры
task migrate            # только применить миграции
```

После `task up` и `task run`:
- Сервер: `http://localhost:8080`
- PostgreSQL: хост `localhost:5433` (в контейнере — 5432)

Для просмотра статуса контейнеров и логов:
```bash
docker compose ps
docker compose logs -f postgres   # при использовании task up + task run
# если сервер запущен в Docker (docker compose up): docker compose logs -f server
```

### Переменные окружения

Скопируйте `.env.example` в `.env` и настройте под свои нужды:
```bash
cp .env.example .env
```

Файл `.env` уже добавлен в `.gitignore` и не будет закоммичен в репозиторий.

Основные переменные сервера (полный список и комментарии — в `.env.example`):
- **БД:** `DATABASE_URL`, опционально `DATABASE_MAX_CONNECTIONS`
- **Безопасность:** `CFGY_MASTER_KEY` (шифрование параметров), `CFGY_JWT_SECRET` (JWT-сессии)
- **Сервер:** `SERVER_HOST`, `SERVER_PORT` (по умолчанию 127.0.0.1:8080)
- **ZID OAuth 2.0:** `ZID_ISSUER`, `ZID_CLIENT_ID`, `ZID_CLIENT_SECRET`, `ZID_REDIRECT_URI`; опционально `ZID_BASE_URL`, `ZID_TLS_INSECURE`, `ZID_CA_FILE`
- **Прочее:** `RUST_LOG`, `ALLOWED_ORIGINS` (CORS), `CFGY_WEB_ROOT`, `ENABLE_HTTPS`, `LOG_FORMAT`

Переопределение при запуске: `CFGY_MASTER_KEY=... CFGY_JWT_SECRET=... task run`

**Для домашнего использования** можно оставить значения по умолчанию — сервер будет работать с предупреждениями.

### Ручной запуск

1. **PostgreSQL** — `task up` или `docker compose up -d postgres`. Порт с хоста: `5433`.
2. **Миграции** — `task migrate` (использует `DATABASE_URL` из `.env`; по умолчанию `postgres://postgres:postgres@localhost:5433/cfgy`). Другой URL: `DATABASE_URL="postgres://..." task migrate`.
3. **Сервер** — `task run` (собирает release, применяет миграции, запускает `cfgy-server`).
4. **Проект для CLI** — в корне проекта: `cfgy init`, затем `cfgy use dev` (или другую configuration). Далее `cfgy get/set/list/run` используют выбранную configuration.
5. **Сборка веб-приложения** — `cd web && npm install && npm run build` (артефакты в `web/build`).

### FreeBSD

Установка сервера как системного сервиса — [freebsd/README.md](./freebsd/README.md).

## Структура репозитория

Монорепозиторий (Cargo workspace):

- `server/` — backend (Axum, Rust, edition 2024)
- `cli/` — CLI-клиент (Rust)
- `web/` — SPA (Svelte 5 + SvelteKit, TypeScript)
- `tests/` — интеграционные smoke-тесты API
- `freebsd/` — установка сервера как системного сервиса на FreeBSD ([freebsd/README.md](./freebsd/README.md))

## Команды Taskfile

Полный список: `task` или `task --list`.

Основные:
- `task run` — собрать сервер (release), применить миграции, запустить сервер локально (нужен PostgreSQL: `task up` или свой инстанс)
- `task web` — запустить веб dev-сервер (SvelteKit, порт 5173)
- `task up` — поднять PostgreSQL в Docker (порт 5433)
- `task down` — остановить контейнеры
- `task migrate` — применить миграции
- `task build` — собрать cfgy-server (release)
- `task build-cli` — собрать CLI (release)
- `task build-web` — собрать веб в `web/build`
- `task generate-secrets` — вывести `CFGY_MASTER_KEY` и `CFGY_JWT_SECRET` (через `cfgy generate-secrets`)
- `task install-cli` — установить CLI в `/usr/local/bin/cfgy` (требуется sudo)
- `task install-freebsd` — установить cfgy-server как системный сервис на FreeBSD (от root на целевом хосте)

Остальное: `cargo build/check/test/clippy/fmt`, `docker compose ps/logs`, `cargo test --bin smoke_tests`.

## Web Dashboard

Дашборд доступен по адресу сервера: `http://localhost:8080` (при `task run`).

Возможности:
- Регистрация, логин (JWT), вход через ZID (OAuth 2.0 Authorization Code), восстановление пароля
- Проекты, configurations (окружения), параметры (ключи–значения) — CRUD (чувствительные параметры хранятся в зашифрованном виде)
- API-токены: создание, просмотр, удаление

### Dev-режим веб-приложения
`task web` запускает SvelteKit dev-сервер на `http://localhost:5173`. API должен быть уже запущен (`task run` в другом терминале). Команда интерактивная — запускайте вручную.

## Smoke-тесты API

Интеграционные тесты в `tests/` проверяют основные сценарии API (регистрация/логин, проекты/configurations/параметры).

Запуск: сервер должен быть доступен на `http://localhost:8080` (`task run`), затем:
```bash
cargo test --bin smoke_tests
```

## Архитектура и документация

- [design.md](./design.md) — архитектура и устройство сервера (Hexagonal Architecture)
- [AGENTS.md](./AGENTS.md) — правила для разработки (AI-агенты, стиль кода, миграции, API)

Лицензия: [LICENSE](./LICENSE).
