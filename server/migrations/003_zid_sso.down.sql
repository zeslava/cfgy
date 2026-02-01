-- Откат миграции: удаление ZID SSO поддержки

-- Удаление индекса для external_identities
DROP INDEX IF EXISTS idx_external_identities_user_id;

-- Удаление таблицы external_identities
DROP TABLE IF EXISTS external_identities CASCADE;

-- Удаление индекса для username
DROP INDEX IF EXISTS idx_users_username_unique;

-- Удаление колонки username
ALTER TABLE users
  DROP COLUMN IF EXISTS username;

-- Возврат password_hash в NOT NULL (требует осторожности, если есть SSO-only пользователи)
-- Внимание: это может вызвать ошибку, если есть пользователи без password_hash
-- В production нужно сначала проверить наличие таких пользователей
ALTER TABLE users
  ALTER COLUMN password_hash SET NOT NULL;
