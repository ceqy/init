-- 创建 OAuth Clients 表

CREATE TABLE IF NOT EXISTS oauth_clients (
    -- 主键
    id UUID PRIMARY KEY,
    
    -- 租户和所有者
    tenant_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    
    -- Client 信息
    name VARCHAR(255) NOT NULL,
    description TEXT,
    client_secret_hash VARCHAR(255),
    client_type VARCHAR(20) NOT NULL CHECK (client_type IN ('Confidential', 'Public')),
    
    -- 授权配置
    grant_types TEXT[] NOT NULL,
    redirect_uris TEXT[] NOT NULL,
    allowed_scopes TEXT[] NOT NULL,
    
    -- Token 生命周期
    access_token_lifetime INTEGER NOT NULL DEFAULT 3600,
    refresh_token_lifetime INTEGER NOT NULL DEFAULT 2592000,
    
    -- 安全配置
    require_pkce BOOLEAN NOT NULL DEFAULT TRUE,
    require_consent BOOLEAN NOT NULL DEFAULT TRUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- 展示信息
    logo_url VARCHAR(500),
    homepage_url VARCHAR(500),
    privacy_policy_url VARCHAR(500),
    terms_of_service_url VARCHAR(500),
    
    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 外键
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_oauth_clients_tenant_id ON oauth_clients(tenant_id);
CREATE INDEX idx_oauth_clients_owner_id ON oauth_clients(owner_id);
CREATE INDEX idx_oauth_clients_is_active ON oauth_clients(is_active) WHERE is_active = TRUE;

-- 启用 Row-Level Security
ALTER TABLE oauth_clients ENABLE ROW LEVEL SECURITY;

-- RLS 策略：只能访问自己租户的 Client
CREATE POLICY oauth_clients_tenant_isolation ON oauth_clients
    USING (tenant_id::text = current_setting('app.current_tenant_id', TRUE));

-- 注释
COMMENT ON TABLE oauth_clients IS 'OAuth2 客户端应用';
COMMENT ON COLUMN oauth_clients.client_type IS 'Client 类型：Confidential（机密）或 Public（公开）';
COMMENT ON COLUMN oauth_clients.grant_types IS '允许的授权类型';
COMMENT ON COLUMN oauth_clients.redirect_uris IS '允许的重定向 URI 列表';
COMMENT ON COLUMN oauth_clients.allowed_scopes IS '允许的 Scope 列表';
COMMENT ON COLUMN oauth_clients.require_pkce IS '是否要求 PKCE（公开客户端强制）';
COMMENT ON COLUMN oauth_clients.require_consent IS '是否要求用户同意';
