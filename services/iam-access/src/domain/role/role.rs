//! 角色实体

use cuba_common::{AuditInfo, TenantId};
use cuba_domain_core::{AggregateRoot, Entity};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::permission::Permission;

/// 角色 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(pub Uuid);

impl RoleId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for RoleId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for RoleId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// 角色实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: RoleId,
    pub tenant_id: TenantId,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub permissions: Vec<Permission>,
    pub audit_info: AuditInfo,
}

impl Role {
    pub fn new(
        tenant_id: TenantId,
        code: String,
        name: String,
        description: Option<String>,
    ) -> Self {
        Self {
            id: RoleId::new(),
            tenant_id,
            code,
            name,
            description,
            is_system: false,
            is_active: true,
            permissions: Vec::new(),
            audit_info: AuditInfo::default(),
        }
    }

    /// 创建系统角色
    pub fn system_role(
        tenant_id: TenantId,
        code: String,
        name: String,
        description: Option<String>,
    ) -> Self {
        let mut role = Self::new(tenant_id, code, name, description);
        role.is_system = true;
        role
    }

    /// 添加权限
    pub fn add_permission(&mut self, permission: Permission) {
        if !self.permissions.iter().any(|p| p.id == permission.id) {
            self.permissions.push(permission);
        }
    }

    /// 移除权限
    pub fn remove_permission(&mut self, permission_id: &PermissionId) {
        self.permissions.retain(|p| &p.id != permission_id);
    }

    /// 检查是否有某个权限
    pub fn has_permission(&self, resource: &str, action: &str) -> bool {
        self.permissions.iter().any(|p| {
            (p.resource == resource || p.resource == "*") && (p.action == action || p.action == "*")
        })
    }

    /// 检查是否有某个权限代码
    pub fn has_permission_code(&self, code: &str) -> bool {
        self.permissions.iter().any(|p| p.code == code)
    }

    /// 激活角色
    pub fn activate(&mut self) {
        self.is_active = true;
    }

    /// 停用角色
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// 更新角色信息
    pub fn update(&mut self, name: String, description: Option<String>) {
        self.name = name;
        self.description = description;
    }
}

impl Entity for Role {
    type Id = RoleId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for Role {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit_info
    }

    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit_info
    }
}

// Re-export PermissionId for convenience
pub use super::permission::PermissionId;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_role() {
        let tenant_id = TenantId::new();
        let role = Role::new(
            tenant_id.clone(),
            "admin".to_string(),
            "Administrator".to_string(),
            Some("System administrator role".to_string()),
        );

        assert_eq!(role.code, "admin");
        assert_eq!(role.name, "Administrator");
        assert!(!role.is_system);
        assert!(role.is_active);
        assert!(role.permissions.is_empty());
    }

    #[test]
    fn test_system_role() {
        let tenant_id = TenantId::new();
        let role = Role::system_role(
            tenant_id,
            "super_admin".to_string(),
            "Super Admin".to_string(),
            None,
        );

        assert!(role.is_system);
    }

    #[test]
    fn test_activate_deactivate() {
        let tenant_id = TenantId::new();
        let mut role = Role::new(tenant_id, "test".to_string(), "Test".to_string(), None);

        role.deactivate();
        assert!(!role.is_active);

        role.activate();
        assert!(role.is_active);
    }
}
