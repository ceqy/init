-- 创建租户表
CREATE TABLE IF NOT EXISTS tenants (
    -- 主键
    id UUID PRIMARY KEY,

    -- 基本信息
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    domain VARCHAR(255) UNIQUE,

    -- 租户设置（JSON）
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,

    -- 状态
    status VARCHAR(50) NOT NULL DEFAULT 'Trial',

    -- 试用和订阅
    trial_ends_at TIMESTAMPTZ,
    subscription_ends_at TIMESTAMPTZ,

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID
);

-- 索引
CREATE INDEX idx_tenants_name ON tenants(name);
CREATE INDEX idx_tenants_domain ON tenants(domain) WHERE domain IS NOT NULL;
CREATE INDEX idx_tenants_status ON tenants(status);
CREATE INDEX idx_tenants_created_at ON tenants(created_at DESC);

-- 注释
COMMENT ON TABLE tenants IS '租户表';
COMMENT ON COLUMN tenants.id IS '租户 ID';
COMMENT ON COLUMN tenants.name IS '租户名称（唯一标识）';
COMMENT ON COLUMN tenants.display_name IS '租户显示名称';
COMMENT ON COLUMN tenants.domain IS '租户域名';
COMMENT ON COLUMN tenants.settings IS '租户设置（JSON）';
COMMENT ON COLUMN tenants.status IS '租户状态：Trial, Active, Suspended, Cancelled';
COMMENT ON COLUMN tenants.trial_ends_at IS '试用到期时间';
COMMENT ON COLUMN tenants.subscription_ends_at IS '订阅到期时间';
