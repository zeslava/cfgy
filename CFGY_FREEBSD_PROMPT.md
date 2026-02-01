# Промпт для cfgy: FreeBSD-сервис и демонизация

Инструкция для настройки сервиса cfgy на FreeBSD (установка, rc.d, демонизация). Соответствует реализации в каталоге `freebsd/`.

---

## 1. Демонизация — ключевые моменты

**Приложение cfgy-server не должно делать демонизацию.**

- В коде: только инициализация и запуск сервера (например `axum::serve(...).await`). Нет `fork()`, отвязки от терминала, перехода в фон.
- Демонизацию выполняет ОС: rc.d-скрипт запускает процесс через `/usr/sbin/daemon`.
- В rc.d: запуск **без** флагов `-p` / `-o` у daemon (чтобы daemon сразу вернул управление). Обёртка: `cd` в рабочую директорию (CWD), подгрузка env из файла, запись PID в pidfile, перенаправление stdout/stderr в лог, затем `exec` бинарника.

Пример строки запуска в rc.d:

```sh
/usr/sbin/daemon -u "${cfgy_server_user}" sh -c \
  "cd \"${cfgy_server_cwd}\" && set -a; . \"${cfgy_server_env_file}\"; set +a; echo \$$ > \"${cfgy_server_pidfile}\"; exec \"${command}\" >> \"${cfgy_server_logfile}\" 2>&1"
```

Итог: cfgy-server — обычный foreground-процесс; фон и отвязка от терминала обеспечиваются обёрткой в rc.d. CWD задаётся, чтобы сервер находил статику Web UI в `$cfgy_server_cwd/web/dist` (или по переменной окружения `CFGY_WEB_ROOT`).

---

## 2. Установка сервиса (кратко)

1. **Бинарник** — `install -m 0755 <binary> /usr/local/bin/cfgy-server`.
2. **Пользователь/группа** — `cfgy` (не cfgy_server); `pw groupadd`, `pw useradd ... -s /usr/sbin/nologin`, домашняя директория `/var/lib/cfgy_server`.
3. **Директории** — `/usr/local/etc/cfgy_server/`, `/var/lib/cfgy_server/`, `/var/log/cfgy_server/`, `/var/run/cfgy_server/`; владелец `cfgy:cfgy`, для etc — `root:cfgy`, chmod 750.
4. **Статика Web UI** — при установке копировать `web/dist` в `/var/lib/cfgy_server/web/dist` (источник: второй аргумент install.sh или по умолчанию `../web/dist` относительно скрипта, если есть).
5. **rc.d** — `install -m 0555 cfgy_server /usr/local/etc/rc.d/cfgy_server`.
6. **Env-файл** — по умолчанию `/usr/local/etc/cfgy_server/cfgy_server.conf`, 0640, `root:cfgy`. Не перезаписывать при повторном запуске install.sh.

---

## 3. RC.D скрипт — нюансы

- Имя сервиса: `cfgy_server` (переменная `name`). Заголовок: `PROVIDE: cfgy_server`, `REQUIRE: NETWORKING DAEMON`, `KEYWORD: nojail`.
- Переменные: `cfgy_server_enable`, `cfgy_server_user`, `cfgy_server_group` (по умолчанию `cfgy`), `cfgy_server_env_file`, `cfgy_server_logfile`, `cfgy_server_pidfile`, `cfgy_server_cwd` (по умолчанию `/var/lib/cfgy_server`). Поддержать устаревший синоним `cfgy_server_config` для env_file.
- **Старт**: проверка наличия env_file; создание директорий для pidfile и logfile; запуск через `daemon -u $user` с обёрткой: `cd $cfgy_server_cwd`, загрузка env, запись PID, перенаправление вывода, `exec` бинарника (без `-p`/`-o` у daemon).
- **Стоп**: чтение PID из pidfile, `kill`, ожидание, при необходимости `kill -9`, удаление pidfile.
- Доп. команды: `status`, `config`, `logs` — через `extra_commands` и соответствующие `*_cmd`.

---

## 4. rc.conf и логи

- Автозапуск: `cfgy_server_enable="YES"` в `/etc/rc.conf`; при необходимости остальные переменные (`cfgy_server_env_file`, `cfgy_server_logfile`, `cfgy_server_pidfile`, `cfgy_server_cwd`).
- Ротация логов: запись в `/etc/newsyslog.conf` (или `/etc/newsyslog.d/`) для `/var/log/cfgy_server/cfgy_server.log`, владелец `cfgy:cfgy`, 644, 7 копий, @daily, Z (сжатие). Пример в `freebsd/newsyslog.conf.sample`.

---

## 5. Конфигурация приложения

- В env-файле: `DATABASE_URL`, `CFGY_MASTER_KEY`, `CFGY_JWT_SECRET`, при необходимости `SERVER_HOST`, `SERVER_PORT`, `RUST_LOG`, `CFGY_WEB_ROOT` (путь к каталогу с index.html, если статика не в `$cfgy_server_cwd/web/dist`), ZID OAuth и др. Шаблон — `freebsd/cfgy_server.conf.sample`.

---

## 6. Важно

- Скрипт установки: проверка root и наличия бинарника; не перезаписывать существующий env-файл. Статику копировать всегда, когда доступен источник (второй аргумент или web/dist в репозитории).
- Права: бинарник 0755, rc.d 0555, env-файл 0640; каталог etc — root:cfgy 750. Пользователь и группа сервиса — `cfgy`.
