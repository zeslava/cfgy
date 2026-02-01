-- Откат миграции: возврат configurations в environments

-- Удалить триггер для configurations
DROP TRIGGER IF EXISTS update_configurations_updated_at ON configurations;

-- Переименовать индексы обратно
DROP INDEX IF EXISTS idx_config_keys_configuration_id;
CREATE INDEX IF NOT EXISTS idx_config_keys_environment_id ON config_keys(environment_id);

DROP INDEX IF EXISTS idx_configurations_project_id;
CREATE INDEX IF NOT EXISTS idx_environments_project_id ON environments(project_id);

-- Обновить внешний ключ обратно
ALTER TABLE config_keys 
  DROP CONSTRAINT IF EXISTS config_keys_configuration_id_fkey;

ALTER TABLE config_keys
  ADD CONSTRAINT config_keys_environment_id_fkey 
  FOREIGN KEY (environment_id) 
  REFERENCES environments(id) 
  ON DELETE CASCADE;

-- Переименовать колонку обратно
ALTER TABLE config_keys RENAME COLUMN configuration_id TO environment_id;

-- Переименовать таблицу обратно
ALTER TABLE configurations RENAME TO environments;

-- Удалить комментарии
COMMENT ON TABLE environments IS NULL;
COMMENT ON COLUMN config_keys.environment_id IS NULL;
