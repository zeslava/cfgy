#!/bin/sh
#
# Установка cfgy-server как системного сервиса на FreeBSD.
# Запуск от root. Бинарник cfgy-server ожидается в текущей директории
# или путь передаётся первым аргументом. Опционально: второй аргумент —
# путь к собранной статике web (каталог web/dist), чтобы раздавать Web UI.
#
# Пример (бинарник + статика):
#   ./install.sh ../target/release/cfgy-server ../web/dist
#

set -e

NAME="cfgy_server"
RUN_AS_USER="cfgy"
BIN_NAME="cfgy-server"
BIN_SOURCE="${1:-./${BIN_NAME}}"
WEB_DIST_SOURCE="${2:-}"
BIN_DEST="/usr/local/bin/${BIN_NAME}"
RC_SCRIPT="$(dirname "$0")/cfgy_server"
RC_DEST="/usr/local/etc/rc.d/${NAME}"
ETC_DIR="/usr/local/etc/${NAME}"
CONF_FILE="${ETC_DIR}/${NAME}.conf"
CONF_SAMPLE="$(dirname "$0")/cfgy_server.conf.sample"
LIB_DIR="/var/lib/${NAME}"
WEB_DIST_DEST="${LIB_DIR}/web/dist"
LOG_DIR="/var/log/${NAME}"
RUN_DIR="/var/run/${NAME}"

if [ "$(id -u)" -ne 0 ]; then
    echo "Ошибка: запустите скрипт от root (id -u = 0)." >&2
    exit 1
fi

if [ ! -f "${BIN_SOURCE}" ]; then
    echo "Ошибка: бинарник не найден: ${BIN_SOURCE}" >&2
    echo "Соберите сервер (cargo build --release -p cfgy-server) и укажите путь, например: ./install.sh /path/to/cfgy-server" >&2
    exit 1
fi

echo "Установка бинарника: ${BIN_SOURCE} -> ${BIN_DEST}"
install -m 0755 "${BIN_SOURCE}" "${BIN_DEST}"

if ! pw groupshow "${RUN_AS_USER}" >/dev/null 2>&1; then
    echo "Создание группы ${RUN_AS_USER}"
    pw groupadd -n "${RUN_AS_USER}"
fi
if ! pw usershow "${RUN_AS_USER}" >/dev/null 2>&1; then
    echo "Создание пользователя ${RUN_AS_USER}"
    pw useradd "${RUN_AS_USER}" -d "${LIB_DIR}" -g "${RUN_AS_USER}" -m -s /usr/sbin/nologin -c "CFGY service"
fi

echo "Создание директорий"
mkdir -p "${ETC_DIR}" "${LIB_DIR}" "${LOG_DIR}" "${RUN_DIR}"
chown "${RUN_AS_USER}:${RUN_AS_USER}" "${LIB_DIR}" "${LOG_DIR}" "${RUN_DIR}"
chown root:"${RUN_AS_USER}" "${ETC_DIR}"
chmod 750 "${ETC_DIR}"

# Опционально: копирование статики Web UI (web/dist) в /var/lib/cfgy_server/web/dist
if [ -n "${WEB_DIST_SOURCE}" ]; then
    if [ ! -d "${WEB_DIST_SOURCE}" ] || [ ! -f "${WEB_DIST_SOURCE}/index.html" ]; then
        echo "Ошибка: каталог статики не найден или в нём нет index.html: ${WEB_DIST_SOURCE}" >&2
        echo "Соберите фронтенд: cd web && npm run build, затем укажите путь к web/dist" >&2
        exit 1
    fi
    echo "Копирование статики Web UI: ${WEB_DIST_SOURCE} -> ${WEB_DIST_DEST}"
    mkdir -p "${WEB_DIST_DEST}"
    cp -R "${WEB_DIST_SOURCE}"/* "${WEB_DIST_DEST}/"
    chown -R "${RUN_AS_USER}:${RUN_AS_USER}" "${LIB_DIR}/web"
fi

if [ ! -f "${CONF_FILE}" ]; then
    echo "Создание шаблона конфига: ${CONF_FILE}"
    if [ -f "${CONF_SAMPLE}" ]; then
        install -m 0640 -o root -g "${RUN_AS_USER}" "${CONF_SAMPLE}" "${CONF_FILE}"
    else
        echo "# Настройте переменные окружения для cfgy-server (DATABASE_URL, CFGY_MASTER_KEY, CFGY_JWT_SECRET и др.)" > "${CONF_FILE}"
        chmod 640 "${CONF_FILE}"
        chown root:"${RUN_AS_USER}" "${CONF_FILE}"
    fi
    echo "Отредактируйте ${CONF_FILE} и включите сервис: ${NAME}_enable=\"YES\" в /etc/rc.conf"
else
    echo "Конфиг уже существует: ${CONF_FILE} (не перезаписываем)"
fi

echo "Установка rc.d скрипта: ${RC_SCRIPT} -> ${RC_DEST}"
install -m 0555 "${RC_SCRIPT}" "${RC_DEST}"

echo ""
echo "Установка завершена."
echo "1. Отредактируйте ${CONF_FILE}"
echo "2. В /etc/rc.conf добавьте: ${NAME}_enable=\"YES\""
echo "3. Запуск: service ${NAME} start"
echo "4. Доп. команды: service ${NAME} status|config|logs"
