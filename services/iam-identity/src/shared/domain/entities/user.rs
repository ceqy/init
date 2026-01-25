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
    // 账户锁定相关
    pub locked_until: Option<DateTime<Utc>>,
    pub lock_reason: Option<String>,
    pub failed_login_count: i32,
    pub last_failed_login_at: Option<DateTime<Utc>>,
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
            locked_until: None,
            lock_reason: None,
            failed_login_count: 0,
            last_failed_login_at: None,
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

    // ========================================================
    // 账户锁定相关方法
    // ========================================================

    /// 记录登录失败
    pub fn record_login_failure(&mut self) {
        self.failed_login_count += 1;
        self.last_failed_login_at = Some(Utc::now());

        // 如果失败次数达到10次，自动锁定账户30分钟
        if self.failed_login_count >= 10 {
            self.lock_account(30, "Too many failed login attempts".to_string());
        }
    }

    /// 清除登录失败记录（登录成功后）
    pub fn clear_login_failures(&mut self) {
        self.failed_login_count = 0;
        self.last_failed_login_at = None;
    }

    /// 锁定账户
    pub fn lock_account(&mut self, minutes: i64, reason: String) {
        self.locked_until = Some(Utc::now() + chrono::Duration::minutes(minutes));
        self.lock_reason = Some(reason);
        self.status = UserStatus::Locked;

        tracing::warn!(
            user_id = %self.id,
            locked_until = ?self.locked_until,
            reason = %self.lock_reason.as_ref().unwrap(),
            "User account locked"
        );
    }

    /// 解锁账户
    pub fn unlock_account(&mut self) {
        self.locked_until = None;
        self.lock_reason = None;
        self.failed_login_count = 0;
        self.last_failed_login_at = None;
        
        // 如果之前是锁定状态，恢复为激活状态
        if self.status == UserStatus::Locked {
            self.status = UserStatus::Active;
        }

        tracing::info!(user_id = %self.id, "User account unlocked");
    }

    /// 检查账户是否被锁定
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    /// 检查账户是否应该自动解锁
    pub fn should_auto_unlock(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() >= locked_until
        } else {
            false
        }
    }

    /// 获取剩余锁定时间（秒）
    pub fn get_lock_remaining_seconds(&self) -> Option<i64> {
        self.locked_until.map(|locked_until| {
            let remaining = locked_until.timestamp() - Utc::now().timestamp();
            remaining.max(0)
        })
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
