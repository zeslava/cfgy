# Установка cfgy-server как сервиса на FreeBSD

Файлы для настройки системного сервиса **cfgy_server** на FreeBSD: rc.d-скрипт, установщик, шаблон конфига и ротация логов. Сервис запускается одним бинарником и при необходимости раздаёт статику Web UI из `/var/lib/cfgy_server/web/dist`.

## Требования

- FreeBSD с установленными базовыми утилитами
- Собранный бинарник `cfgy-server` (например, `cargo build --release -p cfgy-server` → `target/release/cfgy-server`)
- Для Web UI: собранная статика `web/dist` (в каталоге `web/`: `npm ci && npm run build`)

## Быстрая установка

1. Соберите сервер (и при необходимости фронтенд) и перейдите в каталог `freebsd/` репозитория:
   ```sh
   cargo build --release -p cfgy-server
   # Опционально, для Web UI:
   (cd web && npm ci && npm run build)
   cd freebsd
   ```

2. Запустите установку от root. Статика Web UI копируется автоматически из `web/dist` репозитория (если есть). Второй аргумент — путь к `web/dist` (если запускаете не из каталога freebsd/ репо):
   ```sh
   sudo ./install.sh ../target/release/cfgy-server
   ```
   С явным путём к статике:
   ```sh
   sudo ./install.sh ../target/release/cfgy-server /path/to/web/dist
   ```

3. Отредактируйте конфиг и включите сервис:
   ```sh
   sudo vi /usr/local/etc/cfgy_server/cfgy_server.conf
   echo 'cfgy_server_enable="YES"' | sudo tee -a /etc/rc.conf
   ```

4. Запустите сервис:
   ```sh
   sudo service cfgy_server start
   ```

## Файлы

| Файл | Назначение |
|------|------------|
| `cfgy_server` | rc.d-скрипт: start/stop/restart/status/config/logs |
| `install.sh` | Установка бинарника, пользователя, директорий, rc.d и шаблона конфига |
| `cfgy_server.conf.sample` | Шаблон файла окружения (копируется при первом запуске install.sh) |
| `newsyslog.conf.sample` | Пример строки для ротации логов в `/etc/newsyslog.conf` |

## Управление

- `service cfgy_server start` — запуск
- `service cfgy_server stop` — остановка
- `service cfgy_server restart` — перезапуск
- `service cfgy_server status` — статус (PID)
- `service cfgy_server config` — вывод путей и переменных из конфига
- `service cfgy_server logs` — просмотр лога (tail -f)

## Конфигурация

- Файл окружения по умолчанию: `/usr/local/etc/cfgy_server/cfgy_server.conf` (права 0640, root:cfgy).
- В `/etc/rc.conf` можно задать переменные сервиса. Пример (значения по умолчанию можно не указывать):

```sh
cfgy_server_enable="YES"
cfgy_server_user="cfgy"
cfgy_server_group="cfgy"
cfgy_server_env_file="/usr/local/etc/cfgy_server/cfgy_server.conf"
cfgy_server_logfile="/var/log/cfgy_server/cfgy_server.log"
cfgy_server_pidfile="/var/run/cfgy_server/cfgy_server.pid"
cfgy_server_cwd="/var/lib/cfgy_server"
```

  `cfgy_server_cwd` — рабочая директория сервера (по умолчанию `/var/lib/cfgy_server`; оттуда сервер ищет `web/dist`).
- Устаревший синоним для файла конфига: `cfgy_server_config` (поддерживается как fallback для `cfgy_server_env_file`).
- Путь к статике Web UI можно задать переменной `CFGY_WEB_ROOT` в конфиге (каталог с `index.html`, например `/var/lib/cfgy_server/web/dist`).

## Ротация логов

Добавьте строку из `newsyslog.conf.sample` в `/etc/newsyslog.conf` или создайте файл в `/etc/newsyslog.d/` (если система поддерживает include).

## Подробная спецификация

См. [FREEBSD_SERVICE_PROMPT_CFGY.md](../FREEBSD_SERVICE_PROMPT_CFGY.md) в корне репозитория.
