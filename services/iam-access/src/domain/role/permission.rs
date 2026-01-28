//! 权限实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 权限 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermissionId(pub Uuid);

impl PermissionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for PermissionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PermissionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for PermissionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// 权限实体
/// 
/// 权限代表对某个资源执行某个操作的许可
/// 例如: users:read, orders:write, products:delete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: PermissionId,
    /// 权限代码 (唯一标识符，如 "users:read")
    pub code: String,
    /// 权限显示名称
    pub name: String,
    /// 权限描述
    pub description: Option<String>,
    /// 资源标识 (如 "users", "orders")
    pub resource: String,
    /// 操作标识 (如 "read", "write", "delete", "*")
    pub action: String,
    /// 所属模块 (如 "iam", "order", "product")
    pub module: String,
    /// 是否激活
    pub is_active: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Permission {
    pub fn new(
        code: String,
        name: String,
        description: Option<String>,
        resource: String,
        action: String,
        module: String,
    ) -> Self {
        Self {
            id: PermissionId::new(),
            code,
            name,
            description,
            resource,
            action,
            module,
            is_active: true,
            created_at: Utc::now(),
        }
    }

    /// 从资源和操作创建权限代码
    pub fn generate_code(resource: &str, action: &str) -> String {
        format!("{}:{}", resource, action)
    }

    /// 快速创建权限
    pub fn quick(resource: &str, action: &str, module: &str) -> Self {
        let code = Self::generate_code(resource, action);
        let name = format!("{} {}", action.to_uppercase(), resource);
        Self::new(code, name, None, resource.to_string(), action.to_string(), module.to_string())
    }

    /// 激活权限
    pub fn activate(&mut self) {
        self.is_active = true;
    }

    /// 停用权限
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// 检查是否匹配资源和操作
    pub fn matches(&self, resource: &str, action: &str) -> bool {
        (self.resource == resource || self.resource == "*") &&
        (self.action == action || self.action == "*")
    }
}

impl PartialEq for Permission {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Permission {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_permission() {
        let perm = Permission::new(
            "users:read".to_string(),
            "Read Users".to_string(),
            Some("Allows reading user information".to_string()),
            "users".to_string(),
            "read".to_string(),
            "iam".to_string(),
        );

        assert_eq!(perm.code, "users:read");
        assert_eq!(perm.resource, "users");
        assert_eq!(perm.action, "read");
        assert!(perm.is_active);
    }

    #[test]
    fn test_quick_create() {
        let perm = Permission::quick("orders", "write", "order");

        assert_eq!(perm.code, "orders:write");
        assert_eq!(perm.resource, "orders");
        assert_eq!(perm.action, "write");
        assert_eq!(perm.module, "order");
    }

    #[test]
    fn test_matches() {
        let perm = Permission::quick("users", "read", "iam");

        assert!(perm.matches("users", "read"));
        assert!(!perm.matches("users", "write"));
        assert!(!perm.matches("orders", "read"));
    }

    #[test]
    fn test_wildcard_matches() {
        let perm = Permission::quick("users", "*", "iam");

        assert!(perm.matches("users", "read"));
        assert!(perm.matches("users", "write"));
        assert!(perm.matches("users", "delete"));
    }
}
