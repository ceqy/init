//! 策略相关命令定义

use cuba_common::TenantId;

/// 创建策略命令
#[derive(Debug, Clone)]
pub struct CreatePolicyCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub effect: String, // "ALLOW" or "DENY"
    pub subjects: Vec<String>,
    pub resources: Vec<String>,
    pub actions: Vec<String>,
    pub conditions: Option<String>,
    pub priority: i32,
    pub performed_by: String,
}

/// 更新策略命令
#[derive(Debug, Clone)]
pub struct UpdatePolicyCommand {
    pub policy_id: String,
    pub name: String,
    pub description: Option<String>,
    pub effect: String,
    pub subjects: Vec<String>,
    pub resources: Vec<String>,
    pub actions: Vec<String>,
    pub conditions: Option<String>,
    pub priority: i32,
    pub performed_by: String,
}

/// 删除策略命令
#[derive(Debug, Clone)]
pub struct DeletePolicyCommand {
    pub policy_id: String,
    pub performed_by: String,
}

/// 激活/停用策略命令
#[derive(Debug, Clone)]
pub struct SetPolicyActiveCommand {
    pub policy_id: String,
    pub is_active: bool,
    pub performed_by: String,
}
