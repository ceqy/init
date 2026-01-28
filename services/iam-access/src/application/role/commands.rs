//! 角色相关命令定义

use cuba_common::TenantId;

/// 创建角色命令
#[derive(Debug, Clone)]
pub struct CreateRoleCommand {
    pub tenant_id: TenantId,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
}

/// 更新角色命令
#[derive(Debug, Clone)]
pub struct UpdateRoleCommand {
    pub role_id: String,
    pub name: String,
    pub description: Option<String>,
}

/// 删除角色命令
#[derive(Debug, Clone)]
pub struct DeleteRoleCommand {
    pub role_id: String,
}

/// 激活/停用角色命令
#[derive(Debug, Clone)]
pub struct SetRoleActiveCommand {
    pub role_id: String,
    pub is_active: bool,
}

/// 为角色分配权限命令
#[derive(Debug, Clone)]
pub struct AssignPermissionsToRoleCommand {
    pub role_id: String,
    pub permission_ids: Vec<String>,
}

/// 移除角色权限命令
#[derive(Debug, Clone)]
pub struct RemovePermissionsFromRoleCommand {
    pub role_id: String,
    pub permission_ids: Vec<String>,
}

/// 为用户分配角色命令
#[derive(Debug, Clone)]
pub struct AssignRolesToUserCommand {
    pub user_id: String,
    pub tenant_id: TenantId,
    pub role_ids: Vec<String>,
}

/// 移除用户角色命令
#[derive(Debug, Clone)]
pub struct RemoveRolesFromUserCommand {
    pub user_id: String,
    pub tenant_id: TenantId,
    pub role_ids: Vec<String>,
}
