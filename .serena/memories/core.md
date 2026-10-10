# CFGY — Core

Doppler-подобная система управления конфигурацией/параметрами.
Монорепо: один Rust-бинарь (CLI + Axum-сервер) + Svelte 5 SPA.

**Инвариант проекта: весь текст в коде — комментарии, сообщения об ошибках, описания задач Taskfile — на русском.**

## Источник правды по правилам
`AGENTS.md` (подробные правила для агентов) и `CLAUDE.md` в корне — читать их при
любой нетривиальной задаче; они содержат больше деталей, чем эти памятки.

## Карта исходников

```
src/main.rs            # единая точка входа: диспатч CLI vs `serve`
src/cli/               # CLI-клиент (clap): commands, client (HTTP), storage (cfgy.yaml),
                       # config (~/.cfgy/config.yaml + env), crypto, formats
src/server/
  config.rs            # конфиг сервера из env
  application/         # бизнес-логика: auth.rs, projects.rs, parameters.rs
  ports/               # repositories.rs (трейты), crypto.rs (CryptoService), entities.rs
  adapters/api/        # handlers.rs, router.rs, auth.rs (JWT/Bearer/OIDC SSO)
  adapters/persistence/postgres.rs
  adapters/crypto/aes.rs
migrations/            # NNN_name.up.sql + .down.sql (sqlx)
tests/e2e.rs           # e2e против живого сервера
web/                   # SvelteKit SPA, см. `mem:web/core`
freebsd/               # rc.d-скрипт + install.sh для деплоя на FreeBSD
dail/, *.dail          # конфиги dail-jail деплоя (postgresql.conf, pg_hba.conf)
```

## Ключевые инварианты

- Hexagonal (строго): `application/` импортирует **только** из `ports/`; никогда из `adapters/`.
  Все зависимости — трейт-объекты, инжектируются в `main.rs`.
- Доменные сущности — только в `ports/entities.rs` (`#[derive(FromRow)]`).
  Request/Response DTO — только в `adapters/api/handlers.rs`, не в ports.
- Все SQL-запросы параметризованные (sqlx `$1`, `$2`).
- Миграции применяются автоматически при старте сервера (и через `task migrate`).
- CLI работает в двух режимах: **server** (заданы `CFGY_TOKEN` + `CFGY_SERVER_URL`)
  и **local** (файл `cfgy.yaml`, без шифрования).

## Env

`DATABASE_URL` (по умолчанию `postgres://postgres:postgres@localhost:5433/cfgy`),
`CFGY_MASTER_KEY` (hex, 32 байта, AES-GCM), `CFGY_JWT_SECRET`, `RUST_LOG`, `OIDC_*` (OIDC SSO, `ZID_*` — fallback).
`CFGY_MASTER_KEY` обязателен — без него сервер не стартует.
Taskfile подхватывает `.env` (`dotenv:`). Секреты генерируются `task generate-secrets`.

## Дальше

- Стек и версии: `mem:tech_stack`
- Команды сборки/запуска/тестов: `mem:suggested_commands`
- Стиль кода, именование, ошибки: `mem:conventions`
- Что прогонять перед завершением задачи: `mem:task_completion`
- Фронтенд (структура, api-клиент, auth): `mem:web/core`
