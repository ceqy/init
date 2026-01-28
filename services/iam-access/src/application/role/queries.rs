//! 角色相关查询定义

use cuba_common::TenantId;

/// 获取角色详情查询
#[derive(Debug, Clone)]
pub struct GetRoleQuery {
    pub role_id: String,
}

/// 按代码获取角色查询
#[derive(Debug, Clone)]
pub struct GetRoleByCodeQuery {
    pub tenant_id: TenantId,
    pub code: String,
}

/// 列出租户角色查询
#[derive(Debug, Clone)]
pub struct ListRolesQuery {
    pub tenant_id: TenantId,
    pub page: u32,
    pub page_size: u32,
}

/// 搜索角色查询
#[derive(Debug, Clone)]
pub struct SearchRolesQuery {
    pub tenant_id: TenantId,
    pub query: String,
    pub page: u32,
    pub page_size: u32,
}

/// 获取用户角色查询
#[derive(Debug, Clone)]
pub struct GetUserRolesQuery {
    pub user_id: String,
    pub tenant_id: TenantId,
}

/// 获取用户权限查询
#[derive(Debug, Clone)]
pub struct GetUserPermissionsQuery {
    pub user_id: String,
    pub tenant_id: TenantId,
}

/// 检查用户是否有某个权限查询
#[derive(Debug, Clone)]
pub struct CheckUserPermissionQuery {
    pub user_id: String,
    pub tenant_id: TenantId,
    pub permission_code: String,
}
