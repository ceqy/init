//! 租户聚合根

use chrono::{DateTime, Utc};
use cuba_common::{AuditInfo, TenantId};
use cuba_domain_core::{AggregateRoot, Entity};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{PasswordPolicy, TenantSettings};

/// 租户状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantStatus {
    /// 试用中
    Trial,
    /// 激活
    Active,
    /// 暂停
    Suspended,
    /// 已取消
    Cancelled,
}

impl Default for TenantStatus {
    fn default() -> Self {
        Self::Trial
    }
}

/// 租户聚合根
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    /// 租户 ID
    pub id: TenantId,
    /// 租户名称
    pub name: String,
    /// 租户显示名称
    pub display_name: String,
    /// 租户域名（可选）
    pub domain: Option<String>,
    /// 租户设置
    pub settings: TenantSettings,
    /// 租户状态
    pub status: TenantStatus,
    /// 试用到期时间
    pub trial_ends_at: Option<DateTime<Utc>>,
    /// 订阅到期时间
    pub subscription_ends_at: Option<DateTime<Utc>>,
    /// 审计信息
    pub audit_info: AuditInfo,
}

impl Tenant {
    /// 创建新租户
    pub fn new(name: String, display_name: String) -> Result<Self, TenantError> {
        // 验证租户名称
        if name.is_empty() {
            return Err(TenantError::Validation(
                "Tenant name cannot be empty".to_string(),
            ));
        }

        if !Self::is_valid_name(&name) {
            return Err(TenantError::Validation(
                "Tenant name must be alphanumeric and lowercase".to_string(),
            ));
        }

        let id = TenantId::new();
        let trial_ends_at = Some(Utc::now() + chrono::Duration::days(30));

        Ok(Self {
            id,
            name,
            display_name,
            domain: None,
            settings: TenantSettings::default(),
            status: TenantStatus::Trial,
            trial_ends_at,
            subscription_ends_at: None,
            audit_info: AuditInfo::default(),
        })
    }

    /// 验证租户名称格式
    fn is_valid_name(name: &str) -> bool {
        name.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !name.starts_with('-')
            && !name.ends_with('-')
    }

    /// 激活租户
    pub fn activate(&mut self) -> Result<(), TenantError> {
        match self.status {
            TenantStatus::Trial | TenantStatus::Suspended => {
                self.status = TenantStatus::Active;
                Ok(())
            }
            _ => Err(TenantError::InvalidStatusTransition {
                from: self.status.clone(),
                to: TenantStatus::Active,
            }),
        }
    }

    /// 暂停租户
    pub fn suspend(&mut self, reason: String) -> Result<(), TenantError> {
        match self.status {
            TenantStatus::Active | TenantStatus::Trial => {
                self.status = TenantStatus::Suspended;
                tracing::warn!(tenant_id = %self.id, reason = %reason, "Tenant suspended");
                Ok(())
            }
            _ => Err(TenantError::InvalidStatusTransition {
                from: self.status.clone(),
                to: TenantStatus::Suspended,
            }),
        }
    }

    /// 取消租户
    pub fn cancel(&mut self) -> Result<(), TenantError> {
        self.status = TenantStatus::Cancelled;
        Ok(())
    }

    /// 更新租户设置
    pub fn update_settings(&mut self, settings: TenantSettings) {
        self.settings = settings;
    }

    /// 更新密码策略
    pub fn update_password_policy(&mut self, policy: PasswordPolicy) {
        self.settings.password_policy = policy;
    }

    /// 设置用户限制
    pub fn set_user_limit(&mut self, limit: Option<i64>) {
        self.settings.max_users = limit;
    }

    /// 设置域名
    pub fn set_domain(&mut self, domain: String) -> Result<(), TenantError> {
        if !Self::is_valid_domain(&domain) {
            return Err(TenantError::Validation("Invalid domain format".to_string()));
        }
        self.domain = Some(domain);
        Ok(())
    }

    /// 验证域名格式
    fn is_valid_domain(domain: &str) -> bool {
        // 简单的域名验证
        domain.contains('.')
            && domain
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    }

    /// 延长订阅
    pub fn extend_subscription(&mut self, days: i64) {
        let base_date = self.subscription_ends_at.unwrap_or_else(Utc::now);
        self.subscription_ends_at = Some(base_date + chrono::Duration::days(days));
    }

    /// 检查是否激活
    pub fn is_active(&self) -> bool {
        self.status == TenantStatus::Active
    }

    /// 检查是否在试用期
    pub fn is_trial(&self) -> bool {
        self.status == TenantStatus::Trial
    }

    /// 检查试用是否过期
    pub fn is_trial_expired(&self) -> bool {
        if let Some(trial_ends_at) = self.trial_ends_at {
            Utc::now() > trial_ends_at
        } else {
            false
        }
    }

    /// 检查订阅是否过期
    pub fn is_subscription_expired(&self) -> bool {
        if let Some(subscription_ends_at) = self.subscription_ends_at {
            Utc::now() > subscription_ends_at
        } else {
            false
        }
    }

    /// 检查租户是否可用
    pub fn is_available(&self) -> bool {
        match self.status {
            TenantStatus::Active => !self.is_subscription_expired(),
            TenantStatus::Trial => !self.is_trial_expired(),
            _ => false,
        }
    }
}

impl Entity for Tenant {
    type Id = TenantId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for Tenant {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit_info
    }

    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit_info
    }
}

/// 租户错误
#[derive(Debug, thiserror::Error)]
pub enum TenantError {
    #[error("Invalid status transition from {from:?} to {to:?}")]
    InvalidStatusTransition {
        from: TenantStatus,
        to: TenantStatus,
    },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Tenant is not available")]
    NotAvailable,

    #[error("Trial expired")]
    TrialExpired,

    #[error("Subscription expired")]
    SubscriptionExpired,
}
