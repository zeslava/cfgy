-- Migration: add ZID SSO support
--
-- Goals:
-- 1) Allow users without a local password (SSO-only accounts)
-- 2) Add "username" to users
-- 3) Add external identities table for linking ZID user_id -> CFGY user_id
--
-- Notes:
-- - This migration is intentionally additive and backward-compatible.
-- - It does NOT remove existing email/password login; it just makes password optional.

-- 1) Make password_hash nullable for SSO-only users
ALTER TABLE users
  ALTER COLUMN password_hash DROP NOT NULL;

-- 2) Add username column (optional for existing users)
ALTER TABLE users
  ADD COLUMN IF NOT EXISTS username VARCHAR(255);

-- Ensure username is unique when provided (Postgres allows multiple NULLs in a UNIQUE index)
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username_unique
  ON users (username);

-- 3) External identities (provider + subject -> user_id)
-- For ZID:
-- - provider: 'zid'
-- - subject: zid user_id (UUID string)
CREATE TABLE IF NOT EXISTS external_identities (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  provider VARCHAR(64) NOT NULL,
  subject VARCHAR(255) NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (provider, subject)
);

CREATE INDEX IF NOT EXISTS idx_external_identities_user_id
  ON external_identities (user_id);

-- Optional: keep updated_at trigger infrastructure consistent if needed in future.
-- (We don't add updated_at here; created_at is enough for the link table.)
