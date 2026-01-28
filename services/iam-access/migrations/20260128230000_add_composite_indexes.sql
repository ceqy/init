-- 复合索引优化

-- user_roles 复合索引
CREATE INDEX IF NOT EXISTS idx_user_roles_user_tenant ON user_roles(user_id, tenant_id);

-- role_permissions 复合索引
CREATE INDEX IF NOT EXISTS idx_role_permissions_role_perm ON role_permissions(role_id, permission_id);

-- policies 部分索引 (只索引激活的策略)
CREATE INDEX IF NOT EXISTS idx_policies_tenant_active ON policies(tenant_id, priority DESC) WHERE is_active = TRUE;

-- permissions 复合索引用于权限匹配
CREATE INDEX IF NOT EXISTS idx_permissions_resource_action ON permissions(resource, action) WHERE is_active = TRUE;
