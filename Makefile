.PHONY: help run build check test clean up down migrate migrate-check migrate-sync db-setup db-reset cli

# Переменные
PROJECT_NAME := cfgy
SERVER_DIR := server
CLI_DIR := cli

# Определяем команду docker compose (Docker Compose v2 использует "docker compose", v1 - "docker-compose")
# Проверяем наличие docker-compose, если нет - используем docker compose
DOCKER_COMPOSE := $(shell if command -v docker-compose > /dev/null 2>&1; then echo "docker-compose"; else echo "docker compose"; fi)

# Цвета для вывода
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RESET := \033[0m

help: ## Показать справку по командам
	@echo "$(CYAN)Доступные команды:$(RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-15s$(RESET) %s\n", $$1, $$2}'

run: up migrate ## Запустить весь проект (PostgreSQL + сервер)
	@echo "$(CYAN)Запуск CFGY проекта...$(RESET)"
	@echo "$(YELLOW)PostgreSQL уже запущен через docker-compose$(RESET)"
	@cd $(SERVER_DIR) && cargo run

build: ## Собрать все компоненты проекта
	@echo "$(CYAN)Сборка всех компонентов...$(RESET)"
	cargo build

check: ## Проверить код без сборки
	@echo "$(CYAN)Проверка кода...$(RESET)"
	cargo check

test: ## Запустить тесты
	@echo "$(CYAN)Запуск тестов...$(RESET)"
	cargo test

test-smoke: ## Запустить smoke-тесты API
	@echo "$(CYAN)Запуск smoke-тестов...$(RESET)"
	@echo "$(YELLOW)Убедитесь, что сервер запущен на http://localhost:8080$(RESET)"
	cargo test --bin smoke_tests

clean: ## Очистить артефакты сборки
	@echo "$(CYAN)Очистка артефактов сборки...$(RESET)"
	cargo clean

# Docker команды
up: ## Запустить PostgreSQL через docker-compose
	@echo "$(CYAN)Запуск PostgreSQL...$(RESET)"
	$(DOCKER_COMPOSE) up -d
	@echo "$(GREEN)PostgreSQL запущен на порту 5432$(RESET)"

down: ## Остановить и удалить контейнеры PostgreSQL
	@echo "$(CYAN)Остановка PostgreSQL...$(RESET)"
	$(DOCKER_COMPOSE) down

ps: ## Показать статус контейнеров
	$(DOCKER_COMPOSE) ps

logs: ## Показать логи PostgreSQL
	$(DOCKER_COMPOSE) logs -f postgres

# База данных
migrate: ## Применить миграции к базе данных
	@echo "$(CYAN)Применение миграций...$(RESET)"
	@if [ -z "$$DATABASE_URL" ]; then \
		echo "$(YELLOW)Установите DATABASE_URL: export DATABASE_URL='postgres://postgres:postgres@localhost:5432/cfgy'$(RESET)"; \
		exit 1; \
	fi
	@if ! command -v sqlx > /dev/null 2>&1; then \
		echo "$(YELLOW)sqlx-cli не установлен. Установите через: cargo install sqlx-cli --no-default-features --features postgres$(RESET)"; \
		exit 1; \
	fi
	@cd $(SERVER_DIR) && sqlx migrate run --database-url "$$DATABASE_URL" && echo "$(GREEN)Миграции применены успешно$(RESET)" || \
		(echo "$(YELLOW)Внимание: Возможна ошибка при применении миграций$(RESET)"; \
		 echo "$(YELLOW)Если миграции уже применены, это нормально$(RESET)"; \
		 echo "$(YELLOW)Для пересоздания базы данных используйте: make db-reset$(RESET)"; \
		 exit 0)

migrate-check: ## Проверить состояние миграций
	@echo "$(CYAN)Проверка состояния миграций...$(RESET)"
	@if [ -z "$$DATABASE_URL" ]; then \
		echo "$(YELLOW)Установите DATABASE_URL: export DATABASE_URL='postgres://postgres:postgres@localhost:5432/cfgy'$(RESET)"; \
		exit 1; \
	fi
	@if ! command -v sqlx > /dev/null 2>&1; then \
		echo "$(YELLOW)sqlx-cli не установлен. Установите через: cargo install sqlx-cli --no-default-features --features postgres$(RESET)"; \
		exit 1; \
	fi
	@cd $(SERVER_DIR) && sqlx migrate info --database-url "$$DATABASE_URL"

migrate-sync: ## Синхронизировать состояние миграций (для миграций, примененных вручную)
	@echo "$(CYAN)Синхронизация состояния миграций...$(RESET)"
	@if [ -z "$$DATABASE_URL" ]; then \
		echo "$(YELLOW)Установите DATABASE_URL: export DATABASE_URL='postgres://postgres:postgres@localhost:5432/cfgy'$(RESET)"; \
		exit 1; \
	fi
	@if [ ! -f "$(SERVER_DIR)/migrations/001_initial_schema.sql" ]; then \
		echo "$(YELLOW)Файл миграции не найден$(RESET)"; \
		exit 1; \
	fi
	@echo "$(CYAN)Добавление записи о миграции 001_initial_schema...$(RESET)"
	@CHECKSUM=$$(sha256sum $(SERVER_DIR)/migrations/001_initial_schema.sql | cut -d' ' -f1); \
	psql "$$DATABASE_URL" -c "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) VALUES (1, 'initial schema', NOW(), true, decode('$$CHECKSUM', 'hex'), 0) ON CONFLICT (version) DO NOTHING;" && \
	echo "$(GREEN)Миграция синхронизирована$(RESET)" || \
	echo "$(YELLOW)Ошибка синхронизации$(RESET)"

db-setup: up migrate ## Полная настройка БД (запуск PostgreSQL + миграции)
	@echo "$(GREEN)База данных готова!$(RESET)"

db-reset: ## Пересоздать базу данных (остановить, запустить, миграции)
	@echo "$(CYAN)Пересоздание базы данных...$(RESET)"
	@if [ -z "$$DATABASE_URL" ]; then \
		echo "$(YELLOW)Установите DATABASE_URL: export DATABASE_URL='postgres://postgres:postgres@localhost:5432/cfgy'$(RESET)"; \
		exit 1; \
	fi
	@echo "$(YELLOW)ВНИМАНИЕ: Это удалит все данные в базе данных!$(RESET)"
	@echo "$(CYAN)Удаление базы данных...$(RESET)"
	@psql "$$DATABASE_URL" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;" || echo "$(YELLOW)Не удалось удалить схему, возможно база не существует$(RESET)"
	@echo "$(CYAN)Применение миграций...$(RESET)"
	@cd $(SERVER_DIR) && sqlx migrate run --database-url "$$DATABASE_URL"
	@echo "$(GREEN)База данных пересоздана!$(RESET)"

# Разработка
dev: db-setup run ## Полный цикл разработки: настройка БД + запуск сервера

server: ## Запустить только сервер (без PostgreSQL)
	@cd $(SERVER_DIR) && cargo run

cli: ## Собрать CLI
	@cd $(CLI_DIR) && cargo build --release

fmt: ## Форматировать код
	@echo "$(CYAN)Форматирование кода...$(RESET)"
	cargo fmt

clippy: ## Запустить clippy
	@echo "$(CYAN)Запуск clippy...$(RESET)"
	cargo clippy -- -D warnings
