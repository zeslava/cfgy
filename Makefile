.PHONY: help run build check test clean up down migrate migrate-check migrate-sync db-setup db-reset cli web-build

# Переменные
PROJECT_NAME := cfgy
SERVER_DIR := server
CLI_DIR := cli
DATABASE_URL ?= postgres://postgres:postgres@localhost:5433/cfgy

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

run: ## Запустить весь проект через Docker Compose (PostgreSQL + сервер)
	@echo "$(CYAN)Запуск CFGY проекта через Docker Compose...$(RESET)"
	@echo "$(CYAN)Сборка веб-интерфейса (web)...$(RESET)"
	@cd web && npm run build
	@echo "$(GREEN)PostgreSQL будет запущен на порту 5432$(RESET)"
	@echo "$(GREEN)Сервер будет запущен на http://localhost:8080$(RESET)"
	@echo "$(CYAN)Запуск PostgreSQL...$(RESET)"
	@$(DOCKER_COMPOSE) up -d postgres
	@echo "$(CYAN)Ожидание готовности PostgreSQL...$(RESET)"
	@timeout=60; \
	while [ $$timeout -gt 0 ]; do \
		if $(DOCKER_COMPOSE) exec -T postgres pg_isready -U postgres > /dev/null 2>&1; then \
			echo "$(GREEN)PostgreSQL готов!$(RESET)"; \
			break; \
		fi; \
		sleep 1; \
		timeout=$$((timeout - 1)); \
	done; \
	if [ $$timeout -eq 0 ]; then \
		echo "$(YELLOW)Предупреждение: PostgreSQL может быть еще не готов$(RESET)"; \
	fi
	@echo "$(CYAN)Применение миграций...$(RESET)"
	@$(MAKE) migrate || echo "$(YELLOW)Предупреждение: Не удалось применить миграции. Примените их вручную: make migrate$(RESET)"
	@echo "$(CYAN)Запуск сервера в фоновом режиме...$(RESET)"
	@$(DOCKER_COMPOSE) up -d --build server
	@echo "$(GREEN)Сервер запущен в фоновом режиме на http://localhost:8080$(RESET)"
	@echo "$(CYAN)Для просмотра логов используйте: make logs-server$(RESET)"
	@echo "$(CYAN)Для остановки используйте: make down$(RESET)"

build: ## Собрать все компоненты проекта
	@echo "$(CYAN)Сборка всех компонентов...$(RESET)"
	@DATABASE_URL="$(DATABASE_URL)" cargo build

check: ## Проверить код без сборки
	@echo "$(CYAN)Проверка кода...$(RESET)"
	@DATABASE_URL="$(DATABASE_URL)" cargo check

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
up: ## Запустить все сервисы через docker-compose в фоновом режиме
	@echo "$(CYAN)Запуск сервисов через Docker Compose...$(RESET)"
	$(DOCKER_COMPOSE) up -d --build
	@echo "$(GREEN)Сервисы запущены:$(RESET)"
	@echo "$(GREEN)  - PostgreSQL на порту 5432$(RESET)"
	@echo "$(GREEN)  - Сервер на http://localhost:8080$(RESET)"

down: ## Остановить и удалить все контейнеры
	@echo "$(CYAN)Остановка всех контейнеров...$(RESET)"
	$(DOCKER_COMPOSE) down

ps: ## Показать статус контейнеров
	$(DOCKER_COMPOSE) ps

logs: ## Показать логи всех сервисов
	$(DOCKER_COMPOSE) logs -f

logs-postgres: ## Показать логи PostgreSQL
	$(DOCKER_COMPOSE) logs -f postgres

logs-server: ## Показать логи сервера
	$(DOCKER_COMPOSE) logs -f server

# База данных
migrate: ## Применить миграции к базе данных
	@echo "$(CYAN)Применение миграций...$(RESET)"
	@if ! command -v sqlx > /dev/null 2>&1; then \
		echo "$(YELLOW)sqlx-cli не установлен. Установите через: cargo install sqlx-cli --no-default-features --features postgres$(RESET)"; \
		exit 1; \
	fi
	@cd $(SERVER_DIR) && sqlx migrate run --database-url "$(DATABASE_URL)" && echo "$(GREEN)Миграции применены успешно$(RESET)" || \
		(echo "$(YELLOW)Внимание: Возможна ошибка при применении миграций$(RESET)"; \
		 echo "$(YELLOW)Если миграции уже применены, это нормально$(RESET)"; \
		 echo "$(YELLOW)Для пересоздания базы данных используйте: make db-reset$(RESET)"; \
		 exit 0)

migrate-check: ## Проверить состояние миграций
	@echo "$(CYAN)Проверка состояния миграций...$(RESET)"
	@if ! command -v sqlx > /dev/null 2>&1; then \
		echo "$(YELLOW)sqlx-cli не установлен. Установите через: cargo install sqlx-cli --no-default-features --features postgres$(RESET)"; \
		exit 1; \
	fi
	@cd $(SERVER_DIR) && sqlx migrate info --database-url "$(DATABASE_URL)"

migrate-sync: ## Синхронизировать состояние миграций (для миграций, примененных вручную)
	@echo "$(CYAN)Синхронизация состояния миграций...$(RESET)"
	@if [ ! -f "$(SERVER_DIR)/migrations/001_initial_schema.sql" ]; then \
		echo "$(YELLOW)Файл миграции не найден$(RESET)"; \
		exit 1; \
	fi
	@echo "$(CYAN)Добавление записи о миграции 001_initial_schema...$(RESET)"
	@CHECKSUM=$$(sha256sum $(SERVER_DIR)/migrations/001_initial_schema.sql | cut -d' ' -f1); \
	psql "$(DATABASE_URL)" -c "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) VALUES (1, 'initial schema', NOW(), true, decode('$$CHECKSUM', 'hex'), 0) ON CONFLICT (version) DO NOTHING;" && \
	echo "$(GREEN)Миграция синхронизирована$(RESET)" || \
	echo "$(YELLOW)Ошибка синхронизации$(RESET)"

db-setup: up migrate ## Полная настройка БД (запуск PostgreSQL + миграции)
	@echo "$(GREEN)База данных готова!$(RESET)"

db-reset: ## Пересоздать базу данных (остановить, запустить, миграции)
	@echo "$(CYAN)Пересоздание базы данных...$(RESET)"
	@echo "$(YELLOW)ВНИМАНИЕ: Это удалит все данные в базе данных!$(RESET)"
	@echo "$(CYAN)Удаление базы данных...$(RESET)"
	@psql "$(DATABASE_URL)" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;" || echo "$(YELLOW)Не удалось удалить схему, возможно база не существует$(RESET)"
	@echo "$(CYAN)Применение миграций...$(RESET)"
	@cd $(SERVER_DIR) && sqlx migrate run --database-url "$(DATABASE_URL)"
	@echo "$(GREEN)База данных пересоздана!$(RESET)"

# Разработка
dev: db-setup run ## Полный цикл разработки: настройка БД + запуск сервера

dev-local: ## Локальная разработка: PostgreSQL в Docker + сервер локально
	@echo "$(CYAN)Запуск PostgreSQL в Docker...$(RESET)"
	@$(DOCKER_COMPOSE) up -d postgres
	@echo "$(CYAN)Ожидание готовности PostgreSQL...$(RESET)"
	@timeout=60; \
	while [ $$timeout -gt 0 ]; do \
		if $(DOCKER_COMPOSE) exec -T postgres pg_isready -U postgres > /dev/null 2>&1; then \
			echo "$(GREEN)PostgreSQL готов!$(RESET)"; \
			break; \
		fi; \
		sleep 1; \
		timeout=$$((timeout - 1)); \
	done
	@$(MAKE) migrate || true
	@echo "$(CYAN)Запуск сервера локально...$(RESET)"
	@echo "$(GREEN)Сервер будет доступен на http://localhost:8080$(RESET)"
	@echo "$(YELLOW)Нажмите Ctrl+C для остановки$(RESET)"
	@CFGY_MASTER_KEY=dev-master-key CFGY_JWT_SECRET=dev-jwt-secret \
		DATABASE_URL="$(DATABASE_URL)" cargo run --bin cfgy-server

dev-web: ## Запустить веб dev сервер (запустите dev-local в другом терминале)
	@echo "$(CYAN)Запуск веб dev сервера...$(RESET)"
	@echo "$(GREEN)Веб будет доступен на http://localhost:5173$(RESET)"
	@echo "$(CYAN)API на http://localhost:8080 (запустите 'make dev-local' в другом терминале)$(RESET)"
	@echo "$(YELLOW)Нажмите Ctrl+C для остановки$(RESET)"
	@cd web && npm run dev

server: ## Запустить только сервер локально (без Docker, требует запущенный PostgreSQL)
	@echo "$(YELLOW)ВНИМАНИЕ: Запуск сервера локально. Убедитесь, что PostgreSQL запущен.$(RESET)"
	@CFGY_MASTER_KEY=dev-master-key CFGY_JWT_SECRET=dev-jwt-secret \
		DATABASE_URL="$(DATABASE_URL)" cargo run --bin cfgy-server

server-docker: up ## Запустить только сервер в Docker (PostgreSQL должен быть запущен)
	@echo "$(CYAN)Запуск только сервера в Docker...$(RESET)"
	$(DOCKER_COMPOSE) up --build server

cli: ## Собрать CLI
	@cd $(CLI_DIR) && cargo build --release

fmt: ## Форматировать код
	@echo "$(CYAN)Форматирование кода...$(RESET)"
	cargo fmt

clippy: ## Запустить clippy
	@echo "$(CYAN)Запуск clippy...$(RESET)"
	@DATABASE_URL="$(DATABASE_URL)" cargo clippy -- -D warnings
