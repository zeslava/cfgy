# CFGY Copilot Instructions

Detailed project rules live in `AGENTS.md` (Russian). This file is a short English summary.

## Project Overview

**CFGY** is a Doppler-like configuration & parameter management system: **CLI + Axum REST API + Svelte Web Dashboard**, Rust + PostgreSQL, values encrypted server-side with AES-GCM. All code comments and error messages are in Russian.

Single crate, single binary `cfgy` (`src/main.rs`) that dispatches to CLI commands or the server (`cfgy serve`):

- `src/server/` — Axum backend (hexagonal architecture)
- `src/cli/` — CLI client
- `web/` — Svelte 5 + SvelteKit SPA; `web/build/` is embedded into the binary (rust-embed)
- `migrations/` — SQL migrations (auto-applied on server startup)
- `tests/e2e.rs` — E2E tests against a running server

## Architecture: Hexagonal (Ports & Adapters)

```
src/server/
├── application/      # Business logic (auth, projects, parameters) — depends only on ports/
├── ports/
│   ├── repositories.rs   # Repository traits
│   ├── crypto.rs         # CryptoService trait
│   └── entities.rs       # Domain types (User, Project, Configuration, ConfigKey)
├── adapters/
│   ├── api/          # handlers.rs, router.rs, auth.rs (JWT/Bearer/OIDC), static_assets.rs
│   ├── persistence/  # postgres.rs (sqlx)
│   └── crypto/       # aes.rs (AES-GCM)
└── config.rs         # Env-based server config
```

`application/` never imports from `adapters/`; services are generic over port traits and wired up at server startup.

## Data Model

- **User** — email + Argon2 password hash, or OIDC identity linked strictly by (provider, sub)
- **Project** — owned by a User, contains Configurations
- **Configuration** — e.g. "dev", "production"
- **ConfigKey** — parameter; `encrypted_value` stored as BYTEA
- **AccessToken** — long-lived Bearer token for CLI/CI

## Commands

```bash
task up              # PostgreSQL in Docker (port 5433)
task down            # Stop containers
task migrate         # Apply migrations
task run             # Build release + migrate + run server (localhost:8080)
task web             # Svelte dev server (localhost:5173)
task e2e             # E2E tests (builds, migrates, starts/stops server on :8080)
task generate-secrets

cargo check / cargo clippy / cargo fmt / cargo test
```

## Environment Variables

- `DATABASE_URL` — default `postgres://postgres:postgres@localhost:5433/cfgy`
- `CFGY_MASTER_KEY` — hex-encoded 32-byte AES-GCM key, **required** (server refuses to start without it)
- `CFGY_JWT_SECRET` — JWT signing secret (auto-generated with a warning if unset; sessions reset on restart)
- `CFGY_ALLOW_REGISTRATION` — allow email/password registration (by default only the first user)
- `CFGY_WEB_ROOT` — serve web assets from disk instead of the embedded copy
- `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_REDIRECT_URI`, … — OIDC SSO (`ZID_*` accepted as fallback)
- `RUST_LOG` — logging level

## CLI

Modes: **server** (`CFGY_TOKEN` + `CFGY_SERVER_URL` from env or `~/.cfgy/config.yaml`) or **local** (unencrypted `cfgy.yaml`).

```bash
cfgy setup                         # register/login → project → API token → config
cfgy init [-p project] [-c configuration]
cfgy use <configuration>
cfgy set KEY VALUE [-c configuration]
cfgy get KEY [-c configuration] [-f env|json|yaml]
cfgy list [-p project] [-c configuration] [-f env|json|yaml]
cfgy env [-c configuration] [-o [file]]
cfgy delete KEY [-c configuration]
cfgy run -- <command> [args...]
cfgy projects list|create|delete|rename|view|edit
cfgy token list
cfgy config init|show
cfgy completions generate|install
cfgy generate-secrets
cfgy serve
```

## API

Web sessions use the `cfgy_session` cookie (JWT); CLI/CI use `Authorization: Bearer <token>`.

```
GET    /health

POST   /api/auth/register
GET    /api/auth/registration
POST   /api/auth/login
GET    /api/auth/session
GET    /api/auth/me
POST   /api/auth/logout

GET    /auth/oidc/login            # aliases: /oauth/oidc/*, /auth/zid/*, /oauth/zid/*
GET    /auth/oidc/callback

GET    /api/projects
POST   /api/projects
GET    /api/projects/:id
PATCH  /api/projects/:id
DELETE /api/projects/:id

GET    /api/projects/:id/configurations
POST   /api/projects/:id/configurations
DELETE /api/projects/:id/configurations/:config_id

GET    /api/projects/:id/configurations/:config/parameters
PUT    /api/projects/:id/configurations/:config/parameters
DELETE /api/projects/:id/configurations/:config/parameters/:key

GET    /api/tokens
POST   /api/tokens
DELETE /api/tokens/:id
```

## Adding a Feature (server)

1. Extend traits in `ports/repositories.rs` / `ports/crypto.rs`, entities in `ports/entities.rs` (`#[derive(FromRow)]`)
2. Business logic in `application/`
3. Repository implementation in `adapters/persistence/postgres.rs` (parameterized queries only)
4. Handler in `adapters/api/handlers.rs`, route in `adapters/api/router.rs`
5. Schema changes: migration pair `migrations/NNN_name.up.sql` + `.down.sql` (`IF NOT EXISTS` in up)

Conventions: `anyhow::Result` + `.context()`, `async_trait`, `Uuid` IDs, `chrono::DateTime<Utc>`, request/response types in handlers, not in ports.

## Web

- Svelte 5 runes (`$props()`, callback props), SvelteKit with `adapter-static` (SPA, `fallback: index.html`)
- Routes in `web/src/routes/`, components in `web/src/lib/components/`
- API client `web/src/lib/api.ts`, auth `web/src/lib/auth.ts`, toasts `web/src/lib/toast.ts`, global styles `web/src/app.css`
