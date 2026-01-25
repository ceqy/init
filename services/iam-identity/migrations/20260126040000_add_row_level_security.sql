-- 为所有表添加 Row-Level Security (RLS)
-- 确保租户数据隔离

-- ============================================================
-- 启用 RLS
-- ============================================================

-- users 表
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- sessions 表
ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;

-- backup_codes 表
ALTER TABLE backup_codes ENABLE ROW LEVEL SECURITY;

-- password_reset_tokens 表
ALTER TABLE password_reset_tokens ENABLE ROW LEVEL SECURITY;

-- webauthn_credentials 表
ALTER TABLE webauthn_credentials ENABLE ROW LEVEL SECURITY;

-- ============================================================
-- 创建 RLS 策略
-- ============================================================

-- users 表策略
CREATE POLICY users_tenant_isolation ON users
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY users_insert_policy ON users
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY users_update_policy ON users
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY users_delete_policy ON users
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- sessions 表策略
CREATE POLICY sessions_tenant_isolation ON sessions
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY sessions_insert_policy ON sessions
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY sessions_update_policy ON sessions
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY sessions_delete_policy ON sessions
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- backup_codes 表策略
CREATE POLICY backup_codes_tenant_isolation ON backup_codes
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY backup_codes_insert_policy ON backup_codes
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY backup_codes_update_policy ON backup_codes
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY backup_codes_delete_policy ON backup_codes
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- password_reset_tokens 表策略
CREATE POLICY password_reset_tokens_tenant_isolation ON password_reset_tokens
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY password_reset_tokens_insert_policy ON password_reset_tokens
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY password_reset_tokens_update_policy ON password_reset_tokens
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY password_reset_tokens_delete_policy ON password_reset_tokens
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- webauthn_credentials 表策略
CREATE POLICY webauthn_credentials_tenant_isolation ON webauthn_credentials
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY webauthn_credentials_insert_policy ON webauthn_credentials
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY webauthn_credentials_update_policy ON webauthn_credentials
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY webauthn_credentials_delete_policy ON webauthn_credentials
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- ============================================================
-- 创建辅助函数
-- ============================================================

-- 设置当前租户 ID
CREATE OR REPLACE FUNCTION set_current_tenant_id(tenant_uuid UUID)
RETURNS void AS $$
BEGIN
    PERFORM set_config('app.current_tenant_id', tenant_uuid::text, false);
END;
$$ LANGUAGE plpgsql;

-- 获取当前租户 ID
CREATE OR REPLACE FUNCTION get_current_tenant_id()
RETURNS UUID AS $$
BEGIN
    RETURN current_setting('app.current_tenant_id', true)::uuid;
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- 清除当前租户 ID
CREATE OR REPLACE FUNCTION clear_current_tenant_id()
RETURNS void AS $$
BEGIN
    PERFORM set_config('app.current_tenant_id', '', false);
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- 添加索引以优化 RLS 性能
-- ============================================================

-- users 表索引
CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);

-- sessions 表索引
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_id ON sessions(tenant_id);

-- backup_codes 表索引
CREATE INDEX IF NOT EXISTS idx_backup_codes_tenant_id ON backup_codes(tenant_id);

-- password_reset_tokens 表索引
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_tenant_id ON password_reset_tokens(tenant_id);

-- webauthn_credentials 表索引
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_tenant_id ON webauthn_credentials(tenant_id);

-- ============================================================
-- 注释
-- ============================================================

COMMENT ON POLICY users_tenant_isolation ON users IS '用户表租户隔离策略';
COMMENT ON POLICY sessions_tenant_isolation ON sessions IS '会话表租户隔离策略';
COMMENT ON POLICY backup_codes_tenant_isolation ON backup_codes IS '备份码表租户隔离策略';
COMMENT ON POLICY password_reset_tokens_tenant_isolation ON password_reset_tokens IS '密码重置令牌表租户隔离策略';
COMMENT ON POLICY webauthn_credentials_tenant_isolation ON webauthn_credentials IS 'WebAuthn凭证表租户隔离策略';

COMMENT ON FUNCTION set_current_tenant_id(UUID) IS '设置当前会话的租户ID';
COMMENT ON FUNCTION get_current_tenant_id() IS '获取当前会话的租户ID';
COMMENT ON FUNCTION clear_current_tenant_id() IS '清除当前会话的租户ID';
