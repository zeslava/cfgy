# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

CFGY — Doppler-like configuration & parameter management system. Monorepo: Rust backend (CLI + Axum server) + Svelte 5 web dashboard. All comments and error messages in Russian.

## Build & Dev Commands

```bash
# Primary tool: Task (taskfile.dev)
task up              # Start PostgreSQL in Docker (port 5433)
task down            # Stop containers
task migrate         # Apply SQL migrations
task run             # Build release + migrate + run server (localhost:8080)
task web             # Svelte dev server (localhost:5173)
task build           # Build release binary
task build-web       # Build static web assets

# Rust
cargo check          # Type-check
cargo clippy         # Lint
cargo fmt            # Format
cargo test           # Unit tests
task e2e             # E2E tests (tests/e2e.rs; builds, migrates, starts/stops server on :8080)

# Web (from web/)
npm install          # Install deps
npm run dev          # Dev server
npm run build        # Production build → web/build/
```

## Architecture

Unified Rust binary (`src/main.rs`) dispatches to CLI or server mode.

### Server: Hexagonal Architecture (strict)

```
src/server/
├── application/           # Business logic — depends ONLY on ports/ traits
│   ├── auth.rs
│   ├── projects.rs
│   └── parameters.rs
├── ports/                 # Interfaces & domain types
│   ├── repositories.rs    # Repository trait contracts
│   ├── crypto.rs          # CryptoService trait
│   └── entities.rs        # Domain types (User, Project, Configuration, ConfigKey)
└── adapters/              # Concrete implementations
    ├── api/               # Axum HTTP handlers + router
    ├── persistence/       # PostgreSQL (sqlx)
    └── crypto/            # AES-GCM encryption
```

**Rule:** application/ never imports from adapters/. All dependencies injected via trait objects.

### CLI

```
src/cli/
├── commands.rs    # Command handling (clap)
├── client.rs      # HTTP client for server API
├── storage.rs     # cfgy.yaml local storage
├── config.rs      # Config loading (~/.cfgy/config.yaml + env vars)
└── crypto.rs      # Local encryption
```

Two modes: **server** (CFGY_TOKEN + CFGY_SERVER_URL set) or **local** (cfgy.yaml, unencrypted).

### Web Dashboard

Svelte 5 + SvelteKit SPA. File-based routing in `web/src/routes/`. Components in `web/src/lib/components/`. API client in `web/src/lib/api.ts`, auth in `web/src/lib/auth.ts`.

## Database

PostgreSQL 18. Migrations in `migrations/` (auto-applied on server startup).

- Format: `NNN_description.up.sql` + `NNN_description.down.sql`
- Always use `IF NOT EXISTS` in up migrations
- Always parameterized queries via sqlx (`$1`, `$2`, ...)
- Entity types in `ports/entities.rs` with `#[derive(FromRow)]`

## Key Environment Variables

- `DATABASE_URL` — PostgreSQL connection (default: `postgres://postgres:postgres@localhost:5433/cfgy`)
- `CFGY_MASTER_KEY` — hex-encoded 32-byte AES-GCM key (required, server refuses to start without it)
- `CFGY_JWT_SECRET` — JWT signing secret
- `RUST_LOG` — logging level
- `CFGY_ALLOW_REGISTRATION` — email/password registration (default: only the first user)
- `OIDC_*` — OAuth 2.0 / OIDC SSO configuration (`ZID_*` accepted as fallback)

## Adding a New Feature (server-side)

1. Define/extend traits in `ports/repositories.rs` or `ports/crypto.rs`
2. Add entity to `ports/entities.rs` if needed
3. Implement business logic in `application/`
4. Implement repository in `adapters/persistence/postgres.rs`
5. Add handler in `adapters/api/handlers.rs`, route in `adapters/api/router.rs`
6. Create migration pair if schema changes

## Auth

- JWT sessions for web dashboard (cookie: `cfgy_session`)
- Bearer API tokens for CLI/CI
- OIDC SSO (Authorization Code flow), users linked only by (provider, sub)

## Rust Conventions

- `anyhow::Result<T>` with `.context()` for error messages
- `async_trait` for async trait methods
- `Uuid` for IDs, `chrono::DateTime<Utc>` for timestamps
- Request/Response types in handlers, not in ports
