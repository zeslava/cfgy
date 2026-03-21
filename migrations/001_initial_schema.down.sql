-- Откат миграции: удаление базовых таблиц для CFGY

-- Удаление триггеров
DROP TRIGGER IF EXISTS update_config_keys_updated_at ON config_keys;
DROP TRIGGER IF EXISTS update_environments_updated_at ON environments;
DROP TRIGGER IF EXISTS update_projects_updated_at ON projects;
DROP TRIGGER IF EXISTS update_users_updated_at ON users;

-- Удаление функции
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Удаление индексов
DROP INDEX IF EXISTS idx_access_tokens_token_hash;
DROP INDEX IF EXISTS idx_access_tokens_project_id;
DROP INDEX IF EXISTS idx_access_tokens_user_id;
DROP INDEX IF EXISTS idx_config_keys_environment_id;
DROP INDEX IF EXISTS idx_environments_project_id;
DROP INDEX IF EXISTS idx_projects_owner_id;

-- Удаление таблиц (в обратном порядке из-за внешних ключей)
DROP TABLE IF EXISTS access_tokens CASCADE;
DROP TABLE IF EXISTS config_keys CASCADE;
DROP TABLE IF EXISTS environments CASCADE;
DROP TABLE IF EXISTS projects CASCADE;
DROP TABLE IF EXISTS users CASCADE;
