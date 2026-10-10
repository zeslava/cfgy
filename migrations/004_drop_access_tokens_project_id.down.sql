ALTER TABLE access_tokens ADD COLUMN IF NOT EXISTS project_id UUID REFERENCES projects(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS idx_access_tokens_project_id ON access_tokens(project_id);
