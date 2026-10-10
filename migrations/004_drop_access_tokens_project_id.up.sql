-- Токены принадлежат пользователю, привязка к проекту не использовалась
DROP INDEX IF EXISTS idx_access_tokens_project_id;
ALTER TABLE access_tokens DROP COLUMN IF EXISTS project_id;
