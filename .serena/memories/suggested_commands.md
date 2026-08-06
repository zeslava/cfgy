# Команды

Основной раннер — **Task**. `task` без аргументов = список задач. Taskfile читает `.env`.

## Цикл разработки
```bash
task up              # PostgreSQL в Docker (порт 5433) + ожидание готовности
task down            # остановить контейнеры
task migrate         # sqlx migrate run (ignore_error: true)
task run             # build release + migrate + ./target/release/cfgy serve → :8080
task web             # SvelteKit dev-сервер → :5173 (API ожидается на :8080)
task build           # cargo build --release
task build-web       # npm run build → web/build/
task generate-secrets  # CFGY_MASTER_KEY + CFGY_JWT_SECRET в формате env
```

## Rust
```bash
cargo check
cargo clippy
cargo fmt
cargo test           # юнит-тесты
```

## E2E
```bash
task e2e             # scripts/run-e2e.sh: поднимает сервер, гоняет tests/e2e.rs, гасит
```
Не запускать e2e вручную против чужой БД — тесты пишут в БД из `DATABASE_URL`.

## Web (из web/)
```bash
npm install
npm run check        # svelte-check по tsconfig — единственная проверка типов фронта
```

## Деплой (требуют прав, запускать осознанно)
```bash
sudo task install            # /usr/local/bin/cfgy
task install-freebsd         # doas ./freebsd/install.sh
task dail                    # doas dail run cfgy.dail
```
