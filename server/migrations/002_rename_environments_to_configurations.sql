-- Миграция: переименование environments в configurations
-- Переименовываем таблицу и связанные объекты

-- Переименовать таблицу environments в configurations
ALTER TABLE environments RENAME TO configurations;

-- Переименовать колонку environment_id в configuration_id в таблице config_keys
ALTER TABLE config_keys RENAME COLUMN environment_id TO configuration_id;

-- Обновить внешний ключ
ALTER TABLE config_keys 
  DROP CONSTRAINT IF EXISTS config_keys_environment_id_fkey;

ALTER TABLE config_keys
  ADD CONSTRAINT config_keys_configuration_id_fkey 
  FOREIGN KEY (configuration_id) 
  REFERENCES configurations(id) 
  ON DELETE CASCADE;

-- Переименовать индексы
DROP INDEX IF EXISTS idx_environments_project_id;
CREATE INDEX IF NOT EXISTS idx_configurations_project_id ON configurations(project_id);

DROP INDEX IF EXISTS idx_config_keys_environment_id;
CREATE INDEX IF NOT EXISTS idx_config_keys_configuration_id ON config_keys(configuration_id);

-- Обновить триггер для updated_at
DROP TRIGGER IF EXISTS update_environments_updated_at ON configurations;
CREATE TRIGGER update_configurations_updated_at BEFORE UPDATE ON configurations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Обновить комментарии
COMMENT ON TABLE configurations IS 'Конфигурации проекта';
COMMENT ON COLUMN config_keys.configuration_id IS 'ID конфигурации';

