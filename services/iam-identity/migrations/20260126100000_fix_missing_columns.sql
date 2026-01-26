-- Fix missing columns for integration tests

-- Add last_password_change_at to users
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_password_change_at TIMESTAMPTZ;

-- Add public_client to oauth_clients
ALTER TABLE oauth_clients ADD COLUMN IF NOT EXISTS public_client BOOLEAN NOT NULL DEFAULT FALSE;
