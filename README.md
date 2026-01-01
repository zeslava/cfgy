# CFGY

Configuration & Parameter Management с полноценным стеком: **CLI + Server API + Web Dashboard**.

> Примечание по документации: всё описание проекта и инструкции по запуску держим в этом корневом `README.md`.

## Текущий статус

**Этапы 0–3: ✅ завершены (MVP готов)**
- **CLI**: работа в контексте проекта по `.cfgy.yaml` (project + текущая `configuration`) + работа с сервером
- **Backend**: REST API (Axum) + PostgreSQL + миграции
- **Шифрование параметров**: AES-GCM (на сервере используется master key)
- **Web Dashboard** (React + TypeScript): регистрация/логин (JWT), проекты / configurations / параметры (CRUD), API токены

**Этапы 4–6: ⏳ в планах**
- RBAC/команды/коллаборация, аудит
- продвинутые функции (webhooks/интеграции/метрики)
- production-ready (документация, тесты, мониторинг, бэкапы, healthchecks)

Подробнее: [plan.md](./plan.md).

## Структура репозитория

Монорепозиторий (Cargo workspace):

- `server/` — backend сервер (Axum)
- `cli/` — CLI клиент
- `web/` — SPA фронтенд (React + TypeScript, Vite)
- `tests/` — интеграционные smoke-тесты API

## Требования

### Для запуска через Docker (рекомендуется)
- Docker + Docker Compose (v1 `docker-compose` или v2 `docker compose`)
- Node.js + npm (нужно для сборки `web/`, сборка запускается из `Makefile`)
- Rust toolchain (если собираешь/разрабатываешь Rust-компоненты локально)

### Для локального запуска сервера без Docker
- Rust toolchain (edition 2024)
- PostgreSQL
- `sqlx-cli` для миграций:
  - `cargo install sqlx-cli --no-default-features --features postgres`

## Быстрый старт (рекомендуется: Docker Compose + Makefile)

```bash
# посмотреть доступные команды
make help

# запустить PostgreSQL + применить миграции + собрать web + поднять server в Docker
make run

# или "dev" сценарий (настройка БД + run)
make dev

# посмотреть статус контейнеров
make ps

# логи
make logs-server
make logs-postgres

# остановить всё
make down
```

После `make run`:
- Server + Web: `http://localhost:8080`
- PostgreSQL (наружу): `localhost:5433` (в контейнере: `5432`)

### Ежедневный UX (CLI)
CLI работает “в контексте проекта”, который хранится в `.cfgy.yaml` (project + текущая `configuration`).

Минимальный набор команд, который обычно нужен:
```bash
# выбрать configuration по умолчанию для проекта (запишется в .cfgy.yaml)
cfgy use dev

# установить/прочитать параметр в текущей configuration
cfgy set DATABASE_URL postgres://...
cfgy get DATABASE_URL

# список параметров текущей configuration
cfgy list

# запустить команду с параметрами текущей configuration, прокинутыми в env процесса
cfgy run -- <command> [args...]
```

## Переменные окружения

### Сервер (Docker Compose по умолчанию)
В `docker-compose.yml` сервер использует:
- `DATABASE__URL` — строка подключения к PostgreSQL (в Docker-сети)
- `SERVER__HOST`, `SERVER__PORT`
- `CFGY_MASTER_KEY` — master key для шифрования параметров (**в продакшене обязательно менять**)
- `CFGY_JWT_SECRET` — секрет для JWT (**в продакшене обязательно менять**)
- `RUST_LOG` — уровень логирования

Можно переопределять через env при запуске compose, например:
- `CFGY_MASTER_KEY=... CFGY_JWT_SECRET=... make run`

### CLI (режим работы)
Основной источник контекста для CLI — файл проекта `.cfgy.yaml` (project + текущая `configuration`).

Переменные окружения отвечают за подключение к серверу:
- `CFGY_SERVER_URL` — базовый URL сервера, обычно `http://localhost:8080`
- `CFGY_API_TOKEN` — API токен (создаётся через веб-дашборд)

Пример:
```bash
export CFGY_SERVER_URL=http://localhost:8080
export CFGY_API_TOKEN="<token-from-dashboard>"
```

## Ручной запуск (без Makefile)

### 1) Поднять PostgreSQL
В Docker:
```bash
docker compose up -d postgres
# или: docker-compose up -d postgres
```

Порт наружу: `5433`.

### 2) Миграции
Рекомендуемый путь — через `make migrate`, т.к. он использует те же настройки `DATABASE_URL`:

```bash
make migrate
```

По умолчанию `Makefile` использует:
- `DATABASE_URL=postgres://postgres:postgres@localhost:5433/cfgy`

Если у тебя другой URL — передай его:
```bash
DATABASE_URL="postgres://..." make migrate
```

### 3) Запуск сервера локально (без Docker)
Убедись, что PostgreSQL уже запущен и миграции применены, затем:
```bash
make server
```

(Эта команда прокидывает `DATABASE_URL` в `cargo run` для `server/`.)

### 4) Инициализация проекта и выбор configuration
CLI ожидает файл `.cfgy.yaml` (создаётся в директории проекта):

```bash
# в корне твоего проекта
cfgy init

# выбрать configuration по умолчанию (например dev/staging/prod)
cfgy use dev
```

Дальше `cfgy get/set/list/run` будут использовать выбранную `configuration` по умолчанию.

### 5) Сборка фронтенда (если нужно)
Сервер в `make run` сам делает `npm run build` в `web/`.
Если ты запускаешь всё вручную:
```bash
cd web
npm install
npm run build
```

## Команды Makefile (основные)

Смотри полный список:
```bash
make help
```

Ключевые:
- `make run` — сборка `web` + запуск `postgres` + миграции + запуск `server` в Docker (background)
- `make dev` — сценарий разработки (настройка БД + `run`)
- `make up` / `make down` — поднять/остановить контейнеры
- `make migrate` / `make migrate-check` — миграции
- `make db-setup` / `make db-reset` — подготовка/пересоздание БД
- `make logs`, `make logs-server`, `make logs-postgres`
- `make build`, `make check`, `make test`, `make clippy`, `make fmt`
- `make test-smoke` — smoke-тесты API

## Web Dashboard

Дашборд доступен на:
- `http://localhost:8080`

Функции:
- регистрация/логин (JWT)
- проекты: CRUD
- configurations: CRUD
- параметры: CRUD (часть параметров может быть чувствительной, хранение шифруется)
- API токены: создание/просмотр/удаление

### Dev-режим web
В репозитории есть `make dev-web`, который запускает web dev server и предполагает, что API уже запущен в Docker.

Важно: эта команда не подходит для автозапуска в фоне (она интерактивная), поэтому используй её вручную, когда нужно.

## Токены и аутентификация

Есть два типа токенов (концептуально):
- **JWT (session token)** — для веб-сессии (логин в дашборде)
- **API token** — для программного доступа (CLI/CI)

**Важно:** API токены **создаются только через веб-дашборд**.

Порядок:
1. Запусти сервер (`make run`) и открой `http://localhost:8080`
2. Зарегистрируйся/войдите
3. Создай **API token**
4. Экспортируй его для CLI:
   ```bash
   export CFGY_SERVER_URL=http://localhost:8080
   export CFGY_API_TOKEN="<token>"
   ```
5. Инициализируй проект и выбери configuration (один раз на проект/репозиторий):
   ```bash
   cfgy init
   cfgy use dev
   ```
6. Используй короткие команды (`cfgy get/set/list/run`) — они будут применять текущую `configuration` из `.cfgy.yaml`.

## Smoke-тесты API

В `tests/` есть интеграционные smoke-тесты, которые проверяют основные сценарии API (регистрация/логин, проекты/configurations/параметры и т.д.).

### Запуск

1) Убедись, что сервер доступен на `http://localhost:8080` (например, `make run`).

2) Запусти:
```bash
make test-smoke
```

При необходимости можно запускать напрямую:
```bash
cargo test --bin smoke_tests
```

## Архитектура и дизайн

- `design.md` — заметки по архитектуре и устройству сервера