-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100),
    phone VARCHAR(20),
    avatar_url VARCHAR(500),
    tenant_id UUID NOT NULL,
    role_ids TEXT[] DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'PendingVerification',
    language VARCHAR(10) NOT NULL DEFAULT 'zh-CN',
    timezone VARCHAR(50) NOT NULL DEFAULT 'Asia/Shanghai',
    two_factor_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    two_factor_secret VARCHAR(255),
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,
    UNIQUE(username, tenant_id),
    UNIQUE(email, tenant_id)
);

-- 创建索引
CREATE INDEX idx_users_tenant_id ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_status ON users(status);
