//! 角色相关命令定义

use common::TenantId;
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
    /// 验证命令参数
    pub fn validate(&self) -> Result<(), String> {
        if self.code.is_empty() {
            return Err("Role code cannot be empty".to_string());
        }
        if !self
            .code
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(
                "Role code can only contain alphanumeric, underscore, and hyphen".to_string(),
            );
        }
        if self.code.len() > 100 {
            return Err("Role code cannot exceed 100 characters".to_string());
        }
        if self.name.is_empty() {
            return Err("Role name cannot be empty".to_string());
        }
        if self.name.len() > 200 {
            return Err("Role name cannot exceed 200 characters".to_string());
        }
        if let Some(ref desc) = self.description
            && desc.len() > 1000
        {
            return Err("Role description cannot exceed 1000 characters".to_string());
        }
        Ok(())
    }

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
    pub performed_by: Option<Uuid>,
}

/// 为角色分配权限命令
#[derive(Debug, Clone)]
pub struct AssignPermissionsToRoleCommand {
    pub role_id: String,
    pub permission_ids: Vec<String>,
    pub performed_by: Option<Uuid>,
}

/// 移除角色权限命令
#[derive(Debug, Clone)]
pub struct RemovePermissionsFromRoleCommand {
    pub role_id: String,
    pub permission_ids: Vec<String>,
    pub performed_by: Option<Uuid>,
}

/// 为用户分配角色命令
#[derive(Debug, Clone)]
pub struct AssignRolesToUserCommand {
    pub user_id: String,
    pub tenant_id: TenantId,
    pub role_ids: Vec<String>,
    pub performed_by: Option<Uuid>,
}

/// 移除用户角色命令
#[derive(Debug, Clone)]
pub struct RemoveRolesFromUserCommand {
    pub user_id: String,
    pub tenant_id: TenantId,
    pub role_ids: Vec<String>,
    pub performed_by: Option<Uuid>,
}
