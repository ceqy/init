-- 创建授权码表

CREATE TABLE IF NOT EXISTS authorization_codes (
    -- 主键（授权码本身）
    code VARCHAR(255) PRIMARY KEY,
    
    -- 关联
    client_id UUID NOT NULL,
    user_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    
    -- 授权信息
    redirect_uri VARCHAR(500) NOT NULL,
    scopes TEXT[] NOT NULL,
    
    -- PKCE
    code_challenge VARCHAR(255),
    code_challenge_method VARCHAR(10) CHECK (code_challenge_method IN ('S256', 'plain')),
    
    -- 状态
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- 审计
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 外键
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_authorization_codes_client_id ON authorization_codes(client_id);
CREATE INDEX idx_authorization_codes_user_id ON authorization_codes(user_id);
CREATE INDEX idx_authorization_codes_tenant_id ON authorization_codes(tenant_id);
CREATE INDEX idx_authorization_codes_expires_at ON authorization_codes(expires_at);
CREATE INDEX idx_authorization_codes_used ON authorization_codes(used) WHERE used = FALSE;

-- 启用 Row-Level Security
ALTER TABLE authorization_codes ENABLE ROW LEVEL SECURITY;

-- RLS 策略：只能访问自己租户的授权码
CREATE POLICY authorization_codes_tenant_isolation ON authorization_codes
    USING (tenant_id::text = current_setting('app.current_tenant_id', TRUE));

-- 自动清理过期授权码的函数
CREATE OR REPLACE FUNCTION cleanup_expired_authorization_codes()
RETURNS void AS $$
BEGIN
    DELETE FROM authorization_codes
    WHERE expires_at < NOW() - INTERVAL '1 day';
END;
$$ LANGUAGE plpgsql;

-- 注释
COMMENT ON TABLE authorization_codes IS 'OAuth2 授权码（10分钟有效期）';
COMMENT ON COLUMN authorization_codes.code_challenge IS 'PKCE code_challenge';
COMMENT ON COLUMN authorization_codes.code_challenge_method IS 'PKCE 方法：S256（推荐）或 plain';
COMMENT ON COLUMN authorization_codes.used IS '授权码只能使用一次';
