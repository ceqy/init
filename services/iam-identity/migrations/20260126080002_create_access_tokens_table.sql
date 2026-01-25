-- 创建 Access Token 表

CREATE TABLE IF NOT EXISTS access_tokens (
    -- 主键（Token 本身）
    token VARCHAR(500) PRIMARY KEY,
    
    -- 关联
    client_id UUID NOT NULL,
    user_id UUID,  -- Client Credentials 流程可能为 NULL
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
CREATE INDEX idx_access_tokens_client_id ON access_tokens(client_id);
CREATE INDEX idx_access_tokens_user_id ON access_tokens(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_access_tokens_tenant_id ON access_tokens(tenant_id);
CREATE INDEX idx_access_tokens_expires_at ON access_tokens(expires_at);
CREATE INDEX idx_access_tokens_revoked ON access_tokens(revoked) WHERE revoked = FALSE;

-- 启用 Row-Level Security
ALTER TABLE access_tokens ENABLE ROW LEVEL SECURITY;

-- RLS 策略：只能访问自己租户的 Token
CREATE POLICY access_tokens_tenant_isolation ON access_tokens
    USING (tenant_id::text = current_setting('app.current_tenant_id', TRUE));

-- 自动清理过期 Token 的函数
CREATE OR REPLACE FUNCTION cleanup_expired_access_tokens()
RETURNS void AS $$
BEGIN
    DELETE FROM access_tokens
    WHERE expires_at < NOW() - INTERVAL '7 days'
    AND revoked = TRUE;
END;
$$ LANGUAGE plpgsql;

-- 注释
COMMENT ON TABLE access_tokens IS 'OAuth2 Access Token';
COMMENT ON COLUMN access_tokens.user_id IS '用户 ID（Client Credentials 流程为 NULL）';
COMMENT ON COLUMN access_tokens.scopes IS '授权的 Scope 列表';
COMMENT ON COLUMN access_tokens.revoked IS 'Token 是否已撤销';
