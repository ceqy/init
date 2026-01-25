-- 为已存在的业务表添加 tenant_id 字段

-- users 表
ALTER TABLE users ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);

-- sessions 表
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_id ON sessions(tenant_id);

-- password_reset_tokens 表
ALTER TABLE password_reset_tokens ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_tenant_id ON password_reset_tokens(tenant_id);

-- 注释
COMMENT ON COLUMN users.tenant_id IS '租户 ID';
COMMENT ON COLUMN sessions.tenant_id IS '租户 ID';
COMMENT ON COLUMN password_reset_tokens.tenant_id IS '租户 ID';
