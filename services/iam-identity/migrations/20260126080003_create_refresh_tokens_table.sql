-- 创建 Refresh Token 表

CREATE TABLE IF NOT EXISTS refresh_tokens (
    -- 主键（Token 本身）
    token VARCHAR(500) PRIMARY KEY,
    
    -- 关联
    access_token VARCHAR(500) NOT NULL,
    client_id UUID NOT NULL,
    user_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    
    -- 授权信息
    scopes TEXT[] NOT NULL,
    
    -- 状态
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- 审计
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 外键
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_refresh_tokens_access_token ON refresh_tokens(access_token);
CREATE INDEX idx_refresh_tokens_client_id ON refresh_tokens(client_id);
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_tenant_id ON refresh_tokens(tenant_id);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
CREATE INDEX idx_refresh_tokens_revoked ON refresh_tokens(revoked) WHERE revoked = FALSE;

-- 启用 Row-Level Security
ALTER TABLE refresh_tokens ENABLE ROW LEVEL SECURITY;

-- RLS 策略：只能访问自己租户的 Token
CREATE POLICY refresh_tokens_tenant_isolation ON refresh_tokens
    USING (tenant_id::text = current_setting('app.current_tenant_id', TRUE));

-- 自动清理过期 Token 的函数
CREATE OR REPLACE FUNCTION cleanup_expired_refresh_tokens()
RETURNS void AS $$
BEGIN
    DELETE FROM refresh_tokens
    WHERE expires_at < NOW() - INTERVAL '30 days'
    AND revoked = TRUE;
END;
$$ LANGUAGE plpgsql;

-- 注释
COMMENT ON TABLE refresh_tokens IS 'OAuth2 Refresh Token（30天有效期）';
COMMENT ON COLUMN refresh_tokens.access_token IS '关联的 Access Token';
COMMENT ON COLUMN refresh_tokens.scopes IS '授权的 Scope 列表';
COMMENT ON COLUMN refresh_tokens.revoked IS 'Token 是否已撤销';
