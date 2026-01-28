-- 添加复合索引以优化租户特定查询
-- 这些索引对应最常见的查询模式

-- ============================================
-- 1. Users 表复合索引
-- ============================================
-- 删除旧的单列索引（如果存在），因为复合索引可以覆盖这些场景
DROP INDEX IF EXISTS idx_users_status;

-- 添加复合索引用于常见查询：WHERE tenant_id = ? AND email/username = ?
CREATE INDEX IF NOT EXISTS idx_users_tenant_email ON users(tenant_id, email);
CREATE INDEX IF NOT EXISTS idx_users_tenant_username ON users(tenant_id, username);
CREATE INDEX IF NOT EXISTS idx_users_tenant_status ON users(tenant_id, status);

-- 注意：保留以下单列索引用于特定场景
-- idx_users_tenant_id - 用于按租户查询所有用户
-- idx_users_email - 用于跨租户的邮箱唯一性检查（如果需要）
-- idx_users_username - 用于跨租户的用户名唯一性检查（如果需要）

-- ============================================
-- 2. OAuth Clients 表复合索引
-- ============================================
-- 查询模式：WHERE id = ? AND tenant_id = ?
CREATE INDEX IF NOT EXISTS idx_oauth_clients_id_tenant ON oauth_clients(id, tenant_id);

-- ============================================
-- 3. Sessions 表复合索引
-- ============================================
-- 查询模式：WHERE refresh_token_hash = ? AND tenant_id = ?
CREATE INDEX IF NOT EXISTS idx_sessions_token_tenant ON sessions(refresh_token_hash, tenant_id);
-- 查询模式：WHERE user_id = ? AND tenant_id = ?
CREATE INDEX IF NOT EXISTS idx_sessions_user_tenant ON sessions(user_id, tenant_id);

-- ============================================
-- 4. OAuth Tokens 表复合索引
-- ============================================
-- Authorization Codes: WHERE client_id = ? AND tenant_id = ?
CREATE INDEX IF NOT EXISTS idx_authorization_codes_client_tenant ON authorization_codes(client_id, tenant_id);

-- Access Tokens: WHERE client_id = ? AND tenant_id = ?
CREATE INDEX IF NOT EXISTS idx_access_tokens_client_tenant ON access_tokens(client_id, tenant_id);
-- Access Tokens: WHERE user_id = ? AND tenant_id = ?
CREATE INDEX IF NOT EXISTS idx_access_tokens_user_tenant ON access_tokens(user_id, tenant_id);

-- Refresh Tokens: WHERE client_id = ? AND tenant_id = ?
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_client_tenant ON refresh_tokens(client_id, tenant_id);
-- Refresh Tokens: WHERE user_id = ? AND tenant_id = ?
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_tenant ON refresh_tokens(user_id, tenant_id);

-- ============================================
-- 5. Backup Codes 表复合索引
-- ============================================
-- 查询模式：WHERE user_id = ? AND tenant_id = ?
CREATE INDEX IF NOT EXISTS idx_backup_codes_user_tenant ON backup_codes(user_id, tenant_id);

-- ============================================
-- 6. WebAuthn Credentials 表复合索引
-- ============================================
-- 查询模式：WHERE user_id = ? AND tenant_id = ?
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_user_tenant ON webauthn_credentials(user_id, tenant_id);

-- ============================================
-- 7. Email/Phone Verifications 表复合索引
-- ============================================
-- 查询模式：WHERE user_id = ? AND tenant_id = ?
CREATE INDEX IF NOT EXISTS idx_email_verifications_user_tenant_created ON email_verifications(user_id, tenant_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_phone_verifications_user_tenant_created ON phone_verifications(user_id, tenant_id, created_at DESC);

-- ============================================
-- 8. Login Logs 表复合索引（已在原表创建，这里仅记录）
-- ============================================
-- 已存在的复合索引：
-- idx_login_logs_user_tenant_time ON login_logs(user_id, tenant_id, created_at DESC)
-- idx_login_logs_tenant_time ON login_logs(tenant_id, created_at DESC)
