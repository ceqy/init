-- 创建权限表
-- Permissions 是全局的，不按租户隔离

CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY,
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    resource VARCHAR(100) NOT NULL,
    action VARCHAR(50) NOT NULL,
    module VARCHAR(100) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX idx_permissions_code ON permissions(code);
CREATE INDEX idx_permissions_module ON permissions(module);
CREATE INDEX idx_permissions_resource ON permissions(resource);
CREATE INDEX idx_permissions_resource_action ON permissions(resource, action);

-- 添加注释
COMMENT ON TABLE permissions IS '权限表 - 定义系统中所有可用的权限';
COMMENT ON COLUMN permissions.code IS '权限代码，格式: resource:action';
COMMENT ON COLUMN permissions.resource IS '资源标识，如 users, orders';
COMMENT ON COLUMN permissions.action IS '操作标识，如 read, write, delete';
COMMENT ON COLUMN permissions.module IS '所属模块，如 iam, order';
