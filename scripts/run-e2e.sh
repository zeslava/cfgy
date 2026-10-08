#!/usr/bin/env bash
set -euo pipefail

SERVER_PID=""
cleanup() {
  if [ -n "$SERVER_PID" ]; then
    kill "$SERVER_PID" 2>/dev/null || true
    sleep 1
    kill -9 "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

# Убить предыдущий сервер
fuser -k -TERM 8080/tcp 2>/dev/null || true
sleep 1

# Тесты регистрируют много пользователей
CFGY_ALLOW_REGISTRATION=true ./target/release/cfgy serve &
SERVER_PID=$!

for i in $(seq 1 30); do
  curl -sf http://localhost:8080/health > /dev/null 2>&1 && break
  sleep 1
done

cargo test --test e2e -- --nocapture
