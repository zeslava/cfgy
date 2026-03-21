#!/bin/sh
#
# Скрипт установки cfgy-server как сервиса на FreeBSD
# Использование: sudo ./install.sh [путь_к_бинарнику] [путь_к_web/build]
# По умолчанию: ./cfgy-server; статика — из репо web/build, если есть
#

set -e

# Цвета (только если доступен TTY и tput)
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
    RED="$(tput setaf 1)"
    GREEN="$(tput setaf 2)"
    YELLOW="$(tput setaf 3)"
    NC="$(tput sgr0)"
else
    RED=""
    GREEN=""
    YELLOW=""
    NC=""
fi

# Конфигурация
CFGY_USER="cfgy"
CFGY_GROUP="cfgy"
CFGY_HOME="/var/lib/cfgy_server"
CFGY_LOG_DIR="/var/log/cfgy_server"
CFGY_RUN_DIR="/var/run/cfgy_server"
CFGY_ETC_DIR="/usr/local/etc/cfgy_server"
CFGY_NAME="cfgy_server"
BIN_NAME="cfgy"
BIN_PATH="${1:-./${BIN_NAME}}"
WEB_DIST_SOURCE="${2:-}"
if [ -z "${WEB_DIST_SOURCE}" ]; then
    DEFAULT_WEB_DIST="$(cd "$(dirname "$0")/.." 2>/dev/null && pwd)/web/build"
    [ -f "${DEFAULT_WEB_DIST}/index.html" ] && WEB_DIST_SOURCE="${DEFAULT_WEB_DIST}"
fi

echo "${GREEN}=== CFGY FreeBSD Setup ===${NC}"
echo ""

# Проверка прав администратора
if [ "$(id -u)" != "0" ]; then
    echo "${RED}Ошибка: скрипт должен быть запущен с правами root (используйте sudo)${NC}"
    exit 1
fi

# Проверка наличия бинарника
if [ ! -f "$BIN_PATH" ]; then
    echo "${RED}Ошибка: бинарник не найден: $BIN_PATH${NC}"
    echo "Сначала соберите проект: cargo build --release"
    exit 1
fi

echo "📦 Установка бинарника..."
install -m 0755 "$BIN_PATH" "/usr/local/bin/${BIN_NAME}"
echo "${GREEN}✓ Бинарник установлен${NC}"
echo ""

# Создание пользователя и группы
echo "👤 Создание пользователя и группы..."

if pw groupshow "$CFGY_GROUP" >/dev/null 2>&1; then
    echo "  Группа $CFGY_GROUP уже существует"
else
    pw groupadd -n "$CFGY_GROUP"
    echo "  Создана группа $CFGY_GROUP"
fi

if pw usershow "$CFGY_USER" >/dev/null 2>&1; then
    echo "  Пользователь $CFGY_USER уже существует"
else
    pw useradd "$CFGY_USER" \
        -d "$CFGY_HOME" \
        -g "$CFGY_GROUP" \
        -m \
        -s /usr/sbin/nologin \
        -c "CFGY API Service"
    echo "  Создан пользователь $CFGY_USER"
fi
echo "${GREEN}✓ Пользователь и группа готовы${NC}"
echo ""

# Создание директорий
echo "📁 Создание директорий..."

for dir in "$CFGY_ETC_DIR" "$CFGY_HOME" "$CFGY_LOG_DIR" "$CFGY_RUN_DIR"; do
    if [ ! -d "$dir" ]; then
        mkdir -p "$dir"
        echo "  Создана директория: $dir"
    else
        echo "  Директория существует: $dir"
    fi
    chown "$CFGY_USER:$CFGY_GROUP" "$dir"
    chmod 0750 "$dir"
done
chown root:"$CFGY_GROUP" "$CFGY_ETC_DIR"
chmod 750 "$CFGY_ETC_DIR"
echo "${GREEN}✓ Директории готовы${NC}"
echo ""

# Статика Web UI
if [ -n "${WEB_DIST_SOURCE}" ]; then
    echo "🌐 Копирование статики Web UI..."
    if [ ! -d "${WEB_DIST_SOURCE}" ] || [ ! -f "${WEB_DIST_SOURCE}/index.html" ]; then
        echo "${RED}Ошибка: каталог статики не найден или в нём нет index.html: ${WEB_DIST_SOURCE}${NC}"
        echo "Соберите фронтенд: cd web && npm run build"
        exit 1
    fi
    mkdir -p "${CFGY_HOME}/web/build"
    cp -R "${WEB_DIST_SOURCE}"/* "${CFGY_HOME}/web/build/"
    chown -R "$CFGY_USER:$CFGY_GROUP" "${CFGY_HOME}/web"
    echo "${GREEN}✓ Статика установлена в ${CFGY_HOME}/web/build${NC}"
else
    echo "🌐 Статика Web UI не указана (можно добавить позже в ${CFGY_HOME}/web/build)"
fi
echo ""

# Установка rc.d скрипта
echo "🔧 Установка rc.d скрипта..."
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RC_SCRIPT="${SCRIPT_DIR}/cfgy_server"

if [ ! -f "$RC_SCRIPT" ]; then
    echo "${RED}Ошибка: rc.d скрипт не найден: $RC_SCRIPT${NC}"
    exit 1
fi

install -m 0555 "$RC_SCRIPT" "/usr/local/etc/rc.d/${CFGY_NAME}"
echo "${GREEN}✓ rc.d скрипт установлен${NC}"
echo ""

# Создание конфиг-файла окружения
echo "⚙️  Создание конфиг-файла окружения..."
ENV_FILE="${CFGY_ETC_DIR}/${CFGY_NAME}.conf"
CONF_SAMPLE="${SCRIPT_DIR}/cfgy_server.conf.sample"

if [ ! -f "$ENV_FILE" ]; then
    if [ -f "$CONF_SAMPLE" ]; then
        install -m 0640 -o root -g "$CFGY_GROUP" "$CONF_SAMPLE" "$ENV_FILE"
        echo "  Создан конфиг из шаблона: $ENV_FILE"
    else
        cat > "$ENV_FILE" <<'EOF'
# Конфигурация cfgy-server (переменные окружения)
# Права: 0640, владелец root:cfgy

DATABASE_URL=postgres://postgres:postgres@localhost:5432/cfgy
CFGY_MASTER_KEY=your-master-key-minimum-32-characters-long
CFGY_JWT_SECRET=your-jwt-secret-minimum-16-chars
RUST_LOG=info
EOF
        chmod 0640 "$ENV_FILE"
        chown root:"$CFGY_GROUP" "$ENV_FILE"
        echo "  Создан минимальный конфиг: $ENV_FILE"
    fi
else
    echo "  Конфиг-файл уже существует: $ENV_FILE"
    echo "  ${YELLOW}Проверьте и обновите содержимое при необходимости${NC}"
fi
echo "${GREEN}✓ Конфиг-файл готов${NC}"
echo ""

# Итоговый статус
echo "${GREEN}=== Установка завершена ===${NC}"
echo ""
echo "📋 Следующие шаги:"
echo ""
echo "1. Проверьте конфигурацию (обязательно задайте секреты в проде):"
echo "   ${YELLOW}nano ${ENV_FILE}${NC}"
echo "   Укажите: DATABASE_URL, CFGY_MASTER_KEY, CFGY_JWT_SECRET"
echo ""
echo "2. Для автозапуска при загрузке добавьте в /etc/rc.conf:"
echo "   ${YELLOW}echo 'cfgy_server_enable=\"YES\"' >> /etc/rc.conf${NC}"
echo ""
echo "3. Запустите сервис:"
echo "   ${YELLOW}service cfgy_server start${NC}"
echo ""
echo "4. Проверьте статус:"
echo "   ${YELLOW}service cfgy_server status${NC}"
echo ""
echo "5. Просмотр логов:"
echo "   ${YELLOW}service cfgy_server logs${NC}"
echo "   или: ${YELLOW}tail -f /var/log/cfgy_server/cfgy_server.log${NC}"
echo ""
echo "6. Конфиг и переменные окружения:"
echo "   ${YELLOW}service cfgy_server config${NC}"
echo ""
if [ -z "${WEB_DIST_SOURCE}" ]; then
    echo "7. Статика Web UI не установлена. Чтобы добавить:"
    echo "   ${YELLOW}sudo ./install.sh /usr/local/bin/cfgy-server /path/to/web/build${NC}"
    echo ""
fi
