//! 角色相关命令定义

use cuba_common::TenantId;
use uuid::Uuid;

/// 创建角色命令
#[derive(Debug, Clone)]
pub struct CreateRoleCommand {
    pub tenant_id: TenantId,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    /// 执行操作的用户 ID (用于审计)
    pub performed_by: Option<Uuid>,
}

impl CreateRoleCommand {
    /// 将命令转换为角色实体 (移动语义，避免克隆)
    pub fn into_role(self) -> crate::domain::role::Role {
        use crate::domain::role::Role;
        if self.is_system {
            Role::system_role(self.tenant_id, self.code, self.name, self.description)
        } else {
            Role::new(self.tenant_id, self.code, self.name, self.description)
        }
    }
}

/// 更新角色命令
#[derive(Debug, Clone)]
pub struct UpdateRoleCommand {
    pub role_id: String,
    pub name: String,
    pub description: Option<String>,
    /// 执行操作的用户 ID (用于审计)
    pub performed_by: Option<Uuid>,
}

/// 删除角色命令
#[derive(Debug, Clone)]
pub struct DeleteRoleCommand {
    pub role_id: String,
    /// 执行操作的用户 ID (用于审计)
    pub performed_by: Option<Uuid>,
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
