# CFGY Copilot Instructions

## Project Overview

**CFGY** is a configuration & parameter management system with a complete stack: **CLI + Axum REST API + Svelte Web Dashboard**. It's a Doppler-like tool built in Rust with PostgreSQL, managing secrets/parameters across environments with AES-GCM encryption.

**Monorepo Structure** (Cargo workspace):

- `server/` — Backend (Axum framework)
- `cli/` — Command-line client
- `web/` — Svelte 5 + SvelteKit SPA
- `tests/` — Integration smoke tests

## Architecture: Hexagonal (Ports & Adapters)

The server strictly follows hexagonal architecture to isolate business logic from infrastructure:

```
server/src/
├── application/      # Business logic (auth, parameters, projects)
├── ports/            # Interfaces & entity types
│   ├── repositories.rs   # Repository traits (UserRepository, ConfigKeyRepository)
│   ├── crypto.rs         # CryptoService trait
│   └── entities.rs       # Domain types (User, Project, Configuration, ConfigKey)
└── adapters/         # Concrete implementations
    ├── api/          # HTTP handlers (Axum routes)
    ├── persistence/  # PostgreSQL via sqlx
    └── crypto/       # AES-GCM encryption
```

**Key Pattern**: Domain layer (`application/`) **only** depends on traits in `ports/`. No direct database imports—always inject via trait. Tests mock by implementing test doubles of port traits.

## Critical Data Model

**Core Entities** (see `server/migrations/001_initial_schema.sql`):

- **User** — email + password_hash (Argon2)
- **Project** — owned by User, contains multiple Configurations
- **Configuration** — like "production", "dev" (renamed from "environment" in migration 002)
- **ConfigKey** — the actual secret/parameter (encrypted_value as BYTEA)
- **AccessToken** — machine auth (CLI, CI/CD), with optional project scope

**Encryption Strategy**:

- **Passwords**: Argon2 hashing, never encrypted
- **Config values**: AES-GCM encrypted server-side using CFGY_MASTER_KEY env var
- **Storage**: encrypted_value stored as BYTEA in DB

## Build & Test Commands

```bash
# Core workflows (using Taskfile)
task run          # PostgreSQL in Docker + server run locally
task web          # Web dev server (SvelteKit)
task up           # Start PostgreSQL in Docker
task down         # Stop containers
task migrate      # Apply database migrations

# Standard Rust commands
cargo test        # Run tests
cargo test --bin smoke_tests  # Integration tests (requires server on :8080)
cargo build       # Build project
cargo check       # Check code
cargo clippy      # Lint code
cargo fmt         # Format code
```

**Key env vars**:

- `DATABASE_URL` — PostgreSQL connection (default: `postgres://postgres:postgres@localhost:5433/cfgy`)
- `CFGY_MASTER_KEY` — hex-encoded 32-byte key for AES-GCM (docker-compose.yml provides default for dev)
- `CFGY_JWT_SECRET` — secret for JWT tokens
- `RUST_LOG` — logging level

## Dependency Injection Patterns

**Server Dependencies** (injected in main.rs):

```rust
let pool = PgPoolOptions::new()...connect(&config.database.url).await?;
let crypto = AesCryptoService::new(config.crypto.master_key);
let user_repo = PostgresUserRepository::new(pool.clone());
let auth_service = AuthService::new(user_repo.clone(), crypto.clone());
```

Services receive repositories as Arc-wrapped trait objects or via dependency container. **Always use trait objects for cross-layer communication**.

## CLI Context & Commands

The CLI maintains project context in `cfgy.yaml`:

```yaml
project: my-project
configuration: production
```

**Command structure** (defined in `cli/src/commands.rs`):

```bash
cfgy init [project]              # Initialize cfgy.yaml in current directory
cfgy use <configuration>         # Set active configuration (e.g., 'dev', 'production')
cfgy set KEY VALUE [--env ENV]   # Store/update a secret
cfgy get KEY [--env ENV]         # Retrieve a secret
cfgy list [--env ENV]            # List all secrets in configuration
cfgy delete KEY [--env ENV]      # Delete a secret
cfgy run -- <command> [args...]  # Execute command with secrets injected as env vars
cfgy projects create             # Create project (requires server mode)
cfgy token list                  # List API tokens (requires server mode)
```

**Mode detection** — Auto-selects based on env:

- **Server mode** (if `CFGY_TOKEN` + `CFGY_SERVER_URL` are set): All operations hit remote API with encryption
- **Local mode** (default): Stores unencrypted in `cfgy.yaml`

**Key difference**: Local mode stores secrets in plain YAML files (for dev/testing). Server mode encrypts everything server-side with master key.

## API Response Patterns & Authentication

**API Token vs Session Token**:

- **Session Token (JWT)**: Created by `POST /auth/login`, expires after period. Used by Web dashboard UI.
- **API Token**: Long-lived token created in Web dashboard for CLI/CI usage. Sent in `Authorization: Bearer <token>` header.

**Core API Endpoints** (see `tests/smoke_tests.rs` for full request/response types):

```
Auth (no token required):
  POST /auth/register      → { email, password }
  POST /auth/login         → { email, password } → { user_id, email, session_token }

Tokens (requires session/API token):
  POST   /api/tokens       → Create API token
  GET    /api/tokens       → List tokens
  DELETE /api/tokens/{id}  → Revoke token

Projects (requires API token):
  GET    /api/projects              → List user's projects
  POST   /api/projects              → Create project
  GET    /api/projects/{id}         → Get project details
  PUT    /api/projects/{id}         → Update project
  DELETE /api/projects/{id}         → Delete project

Parameters (requires API token):
  POST   /api/projects/{id}/parameters           → Create parameter
  GET    /api/projects/{id}/parameters           → List parameters
  PUT    /api/projects/{id}/parameters/{key}    → Update parameter
  DELETE /api/projects/{id}/parameters/{key}    → Delete parameter
```

All requests except `/auth/*` require `Authorization: Bearer <token>` header.

## ZID SSO Integration (Stage 3)

**ZID** is a CAS-like authentication server for SSO. CFGY integrates with ZID for centralized user authentication.

### How ZID Works

1. User logs in through ZID
2. ZID returns a one-time **ticket**
3. Verify ticket via `/verify` endpoint → get `user_id`, `username`
4. Create session with the verified user

### Integration Method 1: Browser Redirect (Primary)

**Login link:**

```html
<a href="http://zid-host:5555/?return_to=https://yourapp.com/callback">
  Login with ZID
</a>
```

**Callback handler** (`GET /callback?ticket=xxx`):

```rust
#[derive(serde::Deserialize)]
struct ZidVerResult {
    user_id: String,
    username: String,
}

async fn zid_callback(Query(q): Query<std::collections::HashMap<String, String>>) -> Result<Redirect> {
    let ticket = q.get("ticket").ok_or("No ticket")?;

    let client = reqwest::Client::new();
    let res = client
        .post("http://zid-host:5555/verify")
        .json(&serde_json::json!({
            "ticket": ticket,
            "service": "https://yourapp.com/callback"
        }))
        .send()
        .await?;

    let result: ZidVerResult = res.json().await?;

    // Create CFGY session with result.user_id
    create_session(result.user_id).await?;

    Ok(Redirect::to("/dashboard"))
}
```

**Important:** Add your domain to ZID's `TRUSTED_DOMAINS` configuration.

### Integration Method 2: JSON API (SPA/Mobile)

**Step 1:** Send credentials to ZID:

```bash
curl -X POST http://zid-host:5555/login \
  -H "Content-Type: application/json" \
  -d '{"username": "john", "password": "secret"}'
```

Response:

```json
{
  "ticket": "550e8400-e29b-41d4-a716-446655440000",
  "redirect_url": null
}
```

**Step 2:** Verify ticket in application:

```rust
#[derive(serde::Deserialize)]
struct ZidVerResult {
    user_id: String,
    username: String,
}

async fn verify_zid_ticket(ticket: &str) -> Result<ZidVerResult> {
    let client = reqwest::Client::new();

    let response = client
        .post("http://zid-host:5555/verify")
        .json(&serde_json::json!({
            "ticket": ticket,
            "service": "my-service"
        }))
        .send()
        .await?;

    Ok(response.json().await?)
}
```

### Integration Method 3: Telegram Login

ZID supports Telegram authentication:

```bash
curl -X POST http://zid-host:5555/login/telegram \
  -H "Content-Type: application/json" \
  -d '{
    "telegram_id": 123456789,
    "telegram_username": "johndoe",
    "first_name": "John",
    "last_name": "Doe",
    "return_to": "https://yourapp.com/callback"
  }'
```

### ZID Configuration (docker-compose)

```bash
# PostgreSQL (credentials storage)
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=zid
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres

# Redis (sessions & tickets)
REDIS_URL=redis://127.0.0.1:6380

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=5555

# Trusted domains (comma-separated, supports wildcards)
TRUSTED_DOMAINS=localhost,127.0.0.1,*.yourapp.com,yourapp.com

# Storage backends (postgres or redis)
SESSION_STORAGE=redis
TICKET_STORAGE=redis
CREDENTIALS_STORAGE=postgres

# Cookie security (for local HTTP dev: false, for HTTPS: auto or true)
ZID_COOKIE_SECURE=false

# Telegram auto-register
TELEGRAM_AUTO_REGISTER=true
```

### Common ZID Integration Issues

| Issue                        | Cause                                            | Solution                               |
| ---------------------------- | ------------------------------------------------ | -------------------------------------- |
| "Invalid return_to URL"      | Domain not in `TRUSTED_DOMAINS`                  | Add domain to ZID config               |
| SSO cookie not saved locally | `ZID_COOKIE_SECURE=true` with `http://localhost` | Use `ZID_COOKIE_SECURE=false` for HTTP |
| Ticket verification fails    | Ticket is one-time only                          | Request new ticket; don't reuse        |
| Cookie not sent in request   | Missing `credentials: 'include'`                 | Use `credentials: 'include'` in fetch  |

### Health Check

```bash
curl http://zid-host:5555/health
```

## Testing Strategy

- **Unit tests** — Implement mock port traits, test application layer in isolation
- **Smoke tests** (`tests/smoke_tests.rs`) — Full HTTP integration, require running server on `:8080`
- Run smoke tests: `cargo test --bin smoke_tests`

## Common Modifications & Data Flow

**Adding an API endpoint**:

1. Add business logic method to relevant service in `application/`
2. Implement repository method in `ports/repositories.rs` trait
3. Implement repository in `adapters/persistence/postgres.rs`
4. Add HTTP handler in `adapters/api/handlers/` and route in `adapters/api/router.rs`

**Adding a database table**:

1. Create SQL migration in `server/migrations/00X_*.sql`
2. Add entity struct to `ports/entities.rs` with `#[derive(FromRow)]`
3. Extend repository trait in `ports/repositories.rs`
4. Implement in `adapters/persistence/postgres.rs` using sqlx

**Changing encryption**:

1. Modify `adapters/crypto/aes.rs` (implements `CryptoService` trait)
2. Business logic in `application/` never changes — it uses the trait
3. Update key rotation in migrations if needed

**Cross-Component Communication & Data Flow**:

```
Web Dashboard → API Client → Axum Routes → Application Services → Repositories → PostgreSQL
     (Svelte)      (HTTP)      (port/adapters)  (business logic)   (sqlx)       (encryption)
CLI Tool ↔ same flow when in server mode, or local cfgy.yaml when offline
```

**Example: Setting a secret in server mode**:

1. User runs `cfgy set DATABASE_URL postgres://...`
2. CLI sends HTTP POST to `${CFGY_SERVER_URL}/api/projects/{id}/parameters` with auth token
3. Server's Axum handler (in `adapters/api/handlers/`) receives request
4. Handler calls `ParameterService` (in `application/parameters.rs`)
5. Service calls `CryptoService::encrypt()` via port trait (implementation in `adapters/crypto/aes.rs`)
6. Service calls `ParameterRepository::create()` via port trait (implementation in `adapters/persistence/postgres.rs`)
7. Repository executes sqlx query to store encrypted value in DB
8. Response sent back to CLI with success confirmation

**Why this layering matters**:

- Bug in HTTP handler? Affects CLI & Web but not database layer
- Bug in repository? Affects both CLI & Web (server-side)
- Bug in crypto? Only AES-GCM implementation changes; all callers stay the same
- Easy to test: Mock the ports, test application layer without DB/HTTP

## Framework-Specific Details

**Axum** (HTTP):

- Routes defined in `adapters/api/router.rs` with typed extractors
- Handlers in `adapters/api/handlers/` using `AppState` (contains pool, services)
- Use `tower_http::CorsLayer` for CORS (check `router.rs`)

**sqlx** (Database):

- Compile-time checked queries (requires running DB during build)
- Set `DATABASE_URL` during `cargo build` for query macros
- Migrations auto-run on startup in production

**Svelte** (`web/`):

- Svelte 5 + SvelteKit, file-based routing in `web/src/routes/`
- Components in `web/src/lib/components/`, API/auth in `web/src/lib/`
- TypeScript strict mode
- Build output: `web/build/` (adapter-static)

## Web Integration & Svelte Dashboard

The server serves web assets at `/`. After `npm run build` in `web/`, distributes from `web/build/` via `tower_http::fs::ServeDir` and SPA fallback to `index.html`.

**Web conventions**:

- **Svelte 5 + SvelteKit**: File-based routing, component-per-file
- **Styling**: Global `web/src/app.css` and component-scoped styles
- **API calls**: `web/src/lib/api.ts` and `web/src/lib/auth.ts` handle HTTP requests
- **Design compliance**: Rules in `.cursor/rules/design_system_rules.mdc`

## Essential Files & Navigation

| File/Path                          | Purpose                                                        |
| ---------------------------------- | -------------------------------------------------------------- |
| `server/src/main.rs`               | Server initialization, dependency injection setup              |
| `server/src/adapters/api/`         | HTTP handlers organized by domain (auth, projects, parameters) |
| `server/src/application/`          | Pure business logic (no HTTP/DB imports)                       |
| `server/src/ports/`                | Trait definitions & domain types (contracts between layers)    |
| `server/src/adapters/persistence/` | PostgreSQL query implementations                               |
| `server/src/adapters/crypto/`      | AES-GCM encryption implementation                              |
| `cli/src/main.rs`                  | CLI entry point & command dispatcher                           |
| `cli/src/commands.rs`              | Command implementations (set, get, list, run, etc.)            |
| `cli/src/client.rs`                | HTTP client for server API calls                               |
| `cli/src/storage.rs`               | Local `cfgy.yaml` file handling                               |
| `docker-compose.yml`               | PostgreSQL + server container config                           |
| `Taskfile.yml`                     | Task runner for common development commands                     |
| `tests/smoke_tests.rs`             | API integration tests; run with `cargo test --bin smoke_tests`  |
| `web/src/lib/components/`         | Svelte components (Button, Input, Modal, etc.)                  |
| `web/src/lib/api.ts`              | API client; `web/src/lib/auth.ts` for auth                     |

## Debugging Tips

**Server Issues:**

```bash
# View server logs
docker compose logs -f server

# Debug logging
RUST_LOG=debug cargo run -p cfgy-server

# Check database connection
export DATABASE_URL=postgres://postgres:postgres@localhost:5433/cfgy
psql $DATABASE_URL  # Connect directly to inspect tables

# Verify migrations ran
psql $DATABASE_URL -c "\dt"  # List tables
```

**API/CLI Issues:**

```bash
# Test API endpoint directly
curl -H "Authorization: Bearer <token>" http://localhost:8080/api/projects

# Debug CLI in server mode
export CFGY_TOKEN=<your-token>
export CFGY_SERVER_URL=http://localhost:8080
export CFGY_LOG=debug  # If your CLI supports debug logging
cfgy list  # Test command

# Test CLI in local mode (no server needed)
rm cfgy.yaml  # Start fresh
cfgy init my-project
cfgy use dev
cfgy set MY_KEY MY_VALUE
cfgy get MY_KEY  # Should return unencrypted value
cat cfgy.yaml  # Inspect raw file
```

**Database Issues:**

```bash
# Reset database (CAREFUL — deletes data)
sqlx database drop  # Drops database
sqlx database create
sqlx migrate run  # Re-apply all migrations

# Check migration status
sqlx migrate info

# Run SQL directly
psql postgres://postgres:postgres@localhost:5433/cfgy -c "SELECT * FROM users;"
```

## Project Status

- MVP complete (CLI, API, Web Dashboard)
