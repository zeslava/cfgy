# Tech Stack

## Backend (Cargo.toml, единый крейт `cfgy`)
- Rust **edition 2024** (нужен свежий toolchain).
- Axum 0.7 + tower / tower-http (cors, fs, set-header).
- sqlx 0.8 (`runtime-tokio-rustls`, postgres, chrono, uuid) — PostgreSQL 18.
- Крипто: `aes-gcm` (секреты), `argon2`/`password-hash` (пароли), `ring`, `hex`, `base64`.
- `jsonwebtoken` 9 — JWT-сессии.
- clap 4.5 (derive) + clap_complete — CLI.
- reqwest 0.12 — **default-features = false, rustls-tls** (без OpenSSL; не добавлять native-tls).
- anyhow + thiserror, tracing/tracing-subscriber, async-trait, serde/serde_yaml/serde_json,
  uuid v4, chrono, dotenv.

## Frontend (web/)
- Svelte **5** (runes) + SvelteKit 2 + TypeScript 5.9 + Vite 7.
- `@sveltejs/adapter-static` — сборка статической SPA в `web/build/`, отдаётся сервером
  через tower-http.
- Пакетный менеджер: **npm**.

## Инфраструктура
- Task (taskfile.dev) — основной раннер; `Makefile` вторичен.
- Docker Compose — только PostgreSQL, порт **5433** (не 5432).
- `sqlx-cli` нужен для `task migrate`.
- Деплой: FreeBSD rc.d (`freebsd/`) и dail-jail (`cfgy.dail`, `cfgy-pg.dail`).
