-- 创建用户角色关联表
-- 注意: user_id 引用 iam-identity 服务的 users 表，但不创建外键约束（跨服务）

CREATE TABLE IF NOT EXISTS user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 同一用户在同一租户下不能重复分配同一角色
    CONSTRAINT uk_user_roles UNIQUE (user_id, tenant_id, role_id)
);

-- 创建索引
CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_tenant_id ON user_roles(tenant_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX idx_user_roles_user_tenant ON user_roles(user_id, tenant_id);

-- 添加注释
COMMENT ON TABLE user_roles IS '用户角色关联表';
COMMENT ON COLUMN user_roles.user_id IS '用户ID，来自 iam-identity 服务';
