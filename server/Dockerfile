# Стадия сборки фронта (один образ = API + статика из одного бинарника)
FROM node:22-bookworm-slim AS web-builder
WORKDIR /app/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

# Стадия сборки сервера
# Используем latest версию Rust (1.85+ поддерживает edition 2024)
FROM rust:latest as builder

WORKDIR /app

# Установка зависимостей для сборки
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Копирование файлов зависимостей
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/
COPY cli/Cargo.toml ./cli/
COPY tests/Cargo.toml ./tests/

# Создание фиктивных файлов для кеширования зависимостей всех членов workspace
RUN mkdir -p server/src cli/src tests && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "fn main() {}" > cli/src/main.rs && \
    echo "fn main() {}" > tests/smoke_tests.rs && \
    cargo build --release --bin cfgy-server && \
    rm -rf server/src cli/src tests/smoke_tests.rs

# Копирование исходного кода
COPY server/src ./server/src
COPY server/migrations ./server/migrations

# Восстановление фиктивных файлов для других членов workspace (если они были удалены)
# и сборка финального бинарника
# Больше не нужны макросы sqlx, поэтому сборка не требует подключения к БД
RUN mkdir -p cli/src tests && \
    echo "fn main() {}" > cli/src/main.rs && \
    echo "fn main() {}" > tests/smoke_tests.rs && \
    cargo build --release --bin cfgy-server

# Стадия рантайма
FROM debian:bookworm-slim

WORKDIR /app

# Установка зависимостей для рантайма
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Копирование бинарника из стадии сборки
COPY --from=builder /app/target/release/cfgy-server /app/cfgy-server

# Копирование миграций
COPY --from=builder /app/server/migrations ./migrations

# Копирование собранного веб-приложения (SvelteKit → web/build)
COPY --from=web-builder /app/web/build ./web/build

# Порт сервера
EXPOSE 8080

# Запуск сервера
CMD ["./cfgy-server"]

