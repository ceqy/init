-- 创建角色表
-- Roles 按租户隔离

CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,
    
    -- 同一租户内角色代码唯一
    CONSTRAINT uk_roles_tenant_code UNIQUE (tenant_id, code)
);

-- 创建索引
CREATE INDEX idx_roles_tenant_id ON roles(tenant_id);
CREATE INDEX idx_roles_code ON roles(code);
CREATE INDEX idx_roles_is_active ON roles(is_active);

-- 添加注释
COMMENT ON TABLE roles IS '角色表 - 定义租户内的角色';
COMMENT ON COLUMN roles.tenant_id IS '租户ID，用于多租户隔离';
COMMENT ON COLUMN roles.code IS '角色代码，如 admin, editor';
COMMENT ON COLUMN roles.is_system IS '是否系统角色，系统角色不可删除';
