-- Fix OAuth schema issues discovered during integration testing
-- Date: 2026-01-27
-- Related commit: feat(iam-identity): 集成 OAuth 服务并修复数据库列名不匹配
--
-- Issue: Original migrations created scopes columns as TEXT[] (array type),
-- but the Rust code implementation expects TEXT (space-separated string).
-- This migration converts the columns to the correct type.

-- 1. Convert authorization_codes.scopes from TEXT[] to TEXT
DO $$
BEGIN
    -- Check if column is an ARRAY
    IF EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_name = 'authorization_codes' 
        AND column_name = 'scopes' 
        AND data_type = 'ARRAY'
    ) THEN
        ALTER TABLE authorization_codes
        ALTER COLUMN scopes TYPE TEXT
        USING array_to_string(scopes, ' ');
    END IF;
END $$;

-- 2. Convert access_tokens.scopes from TEXT[] to TEXT
-- Note: The column was named 'scopes' in migration but code uses 'scope'
-- First rename if needed, then convert type
DO $$
BEGIN
    -- Rename scopes -> scope if exists
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'access_tokens' AND column_name = 'scopes'
    ) THEN
        ALTER TABLE access_tokens RENAME COLUMN scopes TO scope;
    END IF;

    -- Convert scope ARRAY -> TEXT if it is an ARRAY
    IF EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_name = 'access_tokens' 
        AND column_name = 'scope' 
        AND data_type = 'ARRAY'
    ) THEN
        ALTER TABLE access_tokens
        ALTER COLUMN scope TYPE TEXT
        USING array_to_string(scope, ' ');
    END IF;
END $$;


-- 3. Convert refresh_tokens.scopes from TEXT[] to TEXT
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_name = 'refresh_tokens' 
        AND column_name = 'scopes' 
        AND data_type = 'ARRAY'
    ) THEN
        ALTER TABLE refresh_tokens
        ALTER COLUMN scopes TYPE TEXT
        USING array_to_string(scopes, ' ');
    END IF;
END $$;

-- 4. Add comments for clarity
COMMENT ON COLUMN authorization_codes.scopes IS 'Space-separated list of OAuth scopes (e.g., "read write admin")';
COMMENT ON COLUMN access_tokens.scope IS 'Space-separated list of OAuth scopes (e.g., "read write admin")';
COMMENT ON COLUMN refresh_tokens.scopes IS 'Space-separated list of OAuth scopes (e.g., "read write admin")';
