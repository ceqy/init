//! 策略实体


use cuba_common::{AuditInfo, TenantId};
use cuba_domain_core::{AggregateRoot, Entity};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 策略 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyId(pub Uuid);

impl PolicyId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for PolicyId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PolicyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for PolicyId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// 策略效果
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    /// 允许
    Allow,
    /// 拒绝
    Deny,
}

impl Default for Effect {
    fn default() -> Self {
        Self::Allow
    }
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::Allow => write!(f, "ALLOW"),
            Effect::Deny => write!(f, "DENY"),
        }
    }
}

impl std::str::FromStr for Effect {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ALLOW" => Ok(Effect::Allow),
            "DENY" => Ok(Effect::Deny),
            _ => Err(format!("Unknown effect: {}", s)),
        }
    }
}

/// 策略实体
/// 
/// 策略用于 ABAC (Attribute-Based Access Control)
/// 定义了在什么条件下，什么主体可以对什么资源执行什么操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: PolicyId,
    pub tenant_id: TenantId,
    /// 策略名称
    pub name: String,
    /// 策略描述
    pub description: Option<String>,
    /// 策略效果 (Allow/Deny)
    pub effect: Effect,
    /// 主体匹配模式 (e.g., "user:123", "role:admin", "group:*")
    pub subjects: Vec<String>,
    /// 资源匹配模式 (e.g., "order:*", "product:123")
    pub resources: Vec<String>,
    /// 操作匹配模式 (e.g., "read", "write", "*")
    pub actions: Vec<String>,
    /// 条件表达式 (JSON 格式)
    /// 例如: {"request.owner": "${subject.id}"}
    pub conditions: Option<String>,
    /// 优先级 (数字越大优先级越高，用于解决冲突)
    pub priority: i32,
    /// 是否激活
    pub is_active: bool,
    pub audit_info: AuditInfo,
}

impl Policy {
    pub fn new(
        tenant_id: TenantId,
        name: String,
        effect: Effect,
        subjects: Vec<String>,
        resources: Vec<String>,
        actions: Vec<String>,
    ) -> Self {
        Self {
            id: PolicyId::new(),
            tenant_id,
            name,
            description: None,
            effect,
            subjects,
            resources,
            actions,
            conditions: None,
            priority: 0,
            is_active: true,
            audit_info: AuditInfo::default(),
        }
    }

    /// 创建允许策略
    pub fn allow(
        tenant_id: TenantId,
        name: String,
        subjects: Vec<String>,
        resources: Vec<String>,
        actions: Vec<String>,
    ) -> Self {
        Self::new(tenant_id, name, Effect::Allow, subjects, resources, actions)
    }

    /// 创建拒绝策略
    pub fn deny(
        tenant_id: TenantId,
        name: String,
        subjects: Vec<String>,
        resources: Vec<String>,
        actions: Vec<String>,
    ) -> Self {
        Self::new(tenant_id, name, Effect::Deny, subjects, resources, actions)
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置条件
    pub fn with_conditions(mut self, conditions: String) -> Self {
        self.conditions = Some(conditions);
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// 激活策略
    pub fn activate(&mut self) {
        self.is_active = true;
    }

    /// 停用策略
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// 检查主体是否匹配
    pub fn matches_subject(&self, subject: &str) -> bool {
        self.subjects.iter().any(|s| {
            s == subject || s == "*" || Self::wildcard_match(s, subject)
        })
    }

    /// 检查资源是否匹配
    pub fn matches_resource(&self, resource: &str) -> bool {
        self.resources.iter().any(|r| {
            r == resource || r == "*" || Self::wildcard_match(r, resource)
        })
    }

    /// 检查操作是否匹配
    pub fn matches_action(&self, action: &str) -> bool {
        self.actions.iter().any(|a| {
            a == action || a == "*"
        })
    }

    /// 通配符匹配 (简单实现，支持 * 作为后缀)
    fn wildcard_match(pattern: &str, value: &str) -> bool {
        if let Some(prefix) = pattern.strip_suffix('*') {
            value.starts_with(prefix)
        } else {
            pattern == value
        }
    }

    /// 完整匹配检查
    pub fn matches(&self, subject: &str, resource: &str, action: &str) -> bool {
        self.is_active &&
        self.matches_subject(subject) &&
        self.matches_resource(resource) &&
        self.matches_action(action)
    }
}

impl Entity for Policy {
    type Id = PolicyId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for Policy {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit_info
    }

    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_policy() {
        let tenant_id = TenantId::new();
        let policy = Policy::allow(
            tenant_id,
            "Allow Admin Read Users".to_string(),
            vec!["role:admin".to_string()],
            vec!["users:*".to_string()],
            vec!["read".to_string()],
        );

        assert_eq!(policy.effect, Effect::Allow);
        assert!(policy.is_active);
    }

    #[test]
    fn test_deny_policy() {
        let tenant_id = TenantId::new();
        let policy = Policy::deny(
            tenant_id,
            "Deny Guest Write".to_string(),
            vec!["role:guest".to_string()],
            vec!["*".to_string()],
            vec!["write".to_string(), "delete".to_string()],
        );

        assert_eq!(policy.effect, Effect::Deny);
    }

    #[test]
    fn test_matches_subject() {
        let tenant_id = TenantId::new();
        let policy = Policy::allow(
            tenant_id,
            "Test".to_string(),
            vec!["user:123".to_string(), "role:admin".to_string()],
            vec!["*".to_string()],
            vec!["*".to_string()],
        );

        assert!(policy.matches_subject("user:123"));
        assert!(policy.matches_subject("role:admin"));
        assert!(!policy.matches_subject("user:456"));
    }

    #[test]
    fn test_wildcard_matches() {
        let tenant_id = TenantId::new();
        let policy = Policy::allow(
            tenant_id,
            "Test".to_string(),
            vec!["user:*".to_string()],
            vec!["order:*".to_string()],
            vec!["*".to_string()],
        );

        assert!(policy.matches_subject("user:123"));
        assert!(policy.matches_subject("user:456"));
        assert!(!policy.matches_subject("role:admin"));

        assert!(policy.matches_resource("order:123"));
        assert!(policy.matches_resource("order:456"));
        assert!(!policy.matches_resource("product:123"));
    }

    #[test]
    fn test_full_match() {
        let tenant_id = TenantId::new();
        let policy = Policy::allow(
            tenant_id,
            "Test".to_string(),
            vec!["role:editor".to_string()],
            vec!["article:*".to_string()],
            vec!["read".to_string(), "write".to_string()],
        );

        assert!(policy.matches("role:editor", "article:123", "read"));
        assert!(policy.matches("role:editor", "article:456", "write"));
        assert!(!policy.matches("role:editor", "article:123", "delete"));
        assert!(!policy.matches("role:viewer", "article:123", "read"));
    }
}
