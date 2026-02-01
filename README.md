# CFGY

Configuration & Parameter Management с полноценным стеком: **CLI + Server API + Web Dashboard**.

> Примечание по документации: всё описание проекта и инструкции по запуску держим в этом корневом `README.md`.

## Текущий статус

**Этапы 0–3: ✅ завершены (MVP готов)**
- **CLI**: работа в контексте проекта по `.cfgy.yaml` (project + текущая `configuration`) + работа с сервером
- **Backend**: REST API (Axum) + PostgreSQL + миграции
- **Шифрование параметров**: AES-GCM (на сервере используется master key)
- **Web Dashboard** (Svelte 5 + SvelteKit): регистрация/логин (JWT), проекты / configurations / параметры (CRUD), API токены

**Этапы 4–6: ⏳ в планах**
- RBAC/команды/коллаборация, аудит
- продвинутые функции (webhooks/интеграции/метрики)
- production-ready (документация, тесты, мониторинг, бэкапы, healthchecks)

Подробнее: [plan.md](./plan.md).

## Структура репозитория

Монорепозиторий (Cargo workspace):

- `server/` — backend сервер (Axum)
- `cli/` — CLI клиент
- `web/` — SPA фронтенд (Svelte 5 + SvelteKit)
- `tests/` — интеграционные smoke-тесты API

## Требования

### Для запуска через Docker (рекомендуется)
- Docker + Docker Compose (v1 `docker-compose` или v2 `docker compose`)
- Node.js + npm (нужно для сборки `web/`, артефакты в `web/build`)
- Rust toolchain (если собираешь/разрабатываешь Rust-компоненты локально)
- Task (Taskfile runner): `go install github.com/go-task/task/v3/cmd/task@latest` или через пакетный менеджер

### Для локального запуска сервера без Docker
- Rust toolchain (edition 2024)
- PostgreSQL
- `sqlx-cli` для миграций:
  - `cargo install sqlx-cli --no-default-features --features postgres`

## Быстрый старт (рекомендуется: Docker Compose + Taskfile)

```bash
# посмотреть доступные команды
task

# локальная разработка (PostgreSQL в Docker + сервер локально)
task run

# запустить веб dev сервер (в отдельном терминале)
task web

# запустить PostgreSQL в Docker
task up

# остановить контейнеры
task down

# применить миграции
task migrate
```

После `task run`:
- Server: `http://localhost:8080`
- PostgreSQL (наружу): `localhost:5433` (в контейнере: `5432`)

Для просмотра статуса контейнеров и логов используйте `docker compose` напрямую:
```bash
# статус контейнеров
docker compose ps

# логи сервера
docker compose logs -f server

# логи PostgreSQL
docker compose logs -f postgres
```

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

# сгенерировать секретные ключи для сервера (CFGY_MASTER_KEY, CFGY_JWT_SECRET)
cfgy generate-secrets
# или в формате для shell:
cfgy generate-secrets --format shell
```

## Переменные окружения

### Настройка переменных окружения

Скопируйте `.env.example` в `.env` и настройте под свои нужды:
```bash
cp .env.example .env
```

Файл `.env` уже добавлен в `.gitignore` и не будет закоммичен в репозиторий.

### Сервер (Docker Compose по умолчанию)
В `docker-compose.yml` сервер использует:
- `DATABASE_URL` — строка подключения к PostgreSQL (используется и для миграций, и для сервера)
- `DATABASE_MAX_CONNECTIONS` — максимальное количество подключений к БД (опционально, по умолчанию: 20)
- `SERVER_HOST`, `SERVER_PORT` — хост и порт сервера (опционально)
- `CFGY_MASTER_KEY` — master key для шифрования параметров (рекомендуется установить свой)
- `CFGY_JWT_SECRET` — секрет для JWT (рекомендуется установить свой)
- ZID OAuth 2.0 (вход через ZID): `ZID_ISSUER`, `ZID_CLIENT_ID`, `ZID_CLIENT_SECRET`, `ZID_REDIRECT_URI` — см. `.env.example`
- `RUST_LOG` — уровень логирования

Можно переопределять через env при запуске, например:
- `CFGY_MASTER_KEY=... CFGY_JWT_SECRET=... task run`

**Для домашнего использования:** можно оставить дефолтные значения - система будет работать с предупреждениями.

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

## Ручной запуск

### 1) Поднять PostgreSQL
В Docker:
```bash
task up
# или напрямую: docker compose up -d postgres
```

Порт наружу: `5433`.

### 2) Миграции
Рекомендуемый путь — через `task migrate`:

```bash
task migrate
```

По умолчанию используется переменная окружения `DATABASE_URL`:
- `DATABASE_URL=postgres://postgres:postgres@localhost:5433/cfgy`

Если у тебя другой URL — передай его:
```bash
DATABASE_URL="postgres://..." task migrate
```

### 3) Запуск сервера локально
Убедись, что PostgreSQL уже запущен и миграции применены, затем:
```bash
task run
```

Эта команда запускает сервер локально через `cargo run --bin cfgy-server`.

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
Для production сборки:
```bash
cd web
npm install
npm run build
```

## Команды Taskfile

Смотри полный список:
```bash
task
```

Основные команды:
- `task run` — локальная разработка (PostgreSQL в Docker + сервер локально)
- `task web` — запустить веб (Svelte) dev сервер
- `task up` — запустить PostgreSQL в Docker
- `task down` — остановить контейнеры
- `task migrate` — применить миграции

Для других операций используйте стандартные инструменты:
- `cargo build`, `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt` — стандартные команды Cargo
- `docker compose ps` — статус контейнеров
- `docker compose logs -f [service]` — логи сервисов
- `cargo test --bin smoke_tests` — smoke-тесты API

## Web Dashboard

Дашборд доступен на:
- `http://localhost:8080`

Функции:
- регистрация/логин (JWT), вход через ZID (OAuth 2.0 Authorization Code)
- проекты: CRUD
- configurations: CRUD
- параметры: CRUD (часть параметров может быть чувствительной, хранение шифруется)
- API токены: создание/просмотр/удаление

### Dev-режим web
В репозитории есть `task web`, который запускает web dev server (SvelteKit/Vite) и предполагает, что API уже запущен (через `task run` в другом терминале).

Важно: эта команда интерактивная, поэтому используй её вручную, когда нужно.

## Токены и аутентификация

Есть два типа токенов (концептуально):
- **JWT (session token)** — для веб-сессии (логин в дашборде)
- **API token** — для программного доступа (CLI/CI)

**Важно:** API токены **создаются только через веб-дашборд**.

Порядок:
1. Запусти сервер (`task run`) и открой `http://localhost:8080`
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

1) Убедись, что сервер доступен на `http://localhost:8080` (например, `task run`).

2) Запусти:
```bash
cargo test --bin smoke_tests
```

## Архитектура и дизайн

- `design.md` — заметки по архитектуре и устройству сервера