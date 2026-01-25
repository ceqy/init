//! 用户实体

use chrono::{DateTime, Utc};
use cuba_common::{AuditInfo, TenantId, UserId};
use cuba_domain_core::{AggregateRoot, Entity};
use serde::{Deserialize, Serialize};

use crate::shared::domain::value_objects::{Email, HashedPassword, Username};

/// 用户状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Locked,
    PendingVerification,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::PendingVerification
    }
}

/// 用户实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub email: Email,
    pub password_hash: HashedPassword,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub tenant_id: TenantId,
    pub role_ids: Vec<String>,
    pub status: UserStatus,
    pub language: String,
    pub timezone: String,
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub audit_info: AuditInfo,
}

impl User {
    pub fn new(
        username: Username,
        email: Email,
        password_hash: HashedPassword,
        tenant_id: TenantId,
    ) -> Self {
        Self {
            id: UserId::new(),
            username,
            email,
            password_hash,
            display_name: None,
            phone: None,
            avatar_url: None,
            tenant_id,
            role_ids: Vec::new(),
            status: UserStatus::default(),
            language: "zh-CN".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            two_factor_enabled: false,
            two_factor_secret: None,
            last_login_at: None,
            audit_info: AuditInfo::default(),
        }
    }

    pub fn activate(&mut self) {
        self.status = UserStatus::Active;
    }

    pub fn deactivate(&mut self) {
        self.status = UserStatus::Inactive;
    }

    pub fn lock(&mut self) {
        self.status = UserStatus::Locked;
    }

    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }

    pub fn record_login(&mut self) {
        self.last_login_at = Some(Utc::now());
    }

    pub fn enable_2fa(&mut self, secret: String) {
        self.two_factor_enabled = true;
        self.two_factor_secret = Some(secret);
    }

    pub fn disable_2fa(&mut self) {
        self.two_factor_enabled = false;
        self.two_factor_secret = None;
    }

    pub fn update_password(&mut self, password_hash: HashedPassword) {
        self.password_hash = password_hash;
    }

    pub fn add_role(&mut self, role_id: String) {
        if !self.role_ids.contains(&role_id) {
            self.role_ids.push(role_id);
        }
    }

    pub fn remove_role(&mut self, role_id: &str) {
        self.role_ids.retain(|r| r != role_id);
    }
}

impl Entity for User {
    type Id = UserId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for User {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit_info
    }

    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit_info
    }
}
