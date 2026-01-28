//! 用户实体

use chrono::{DateTime, Utc};
use cuba_common::{AuditInfo, TenantId, UserId};
use cuba_domain_core::{AggregateRoot, Entity};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{Email, HashedPassword, Username};

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

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatus::Active => write!(f, "Active"),
            UserStatus::Inactive => write!(f, "Inactive"),
            UserStatus::Locked => write!(f, "Locked"),
            UserStatus::PendingVerification => write!(f, "PendingVerification"),
        }
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
    // 邮箱验证
    pub email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    // 手机验证
    pub phone_verified: bool,
    pub phone_verified_at: Option<DateTime<Utc>>,
    pub last_password_change_at: Option<DateTime<Utc>>,
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
            email_verified: false,
            email_verified_at: None,
            phone_verified: false,
            phone_verified_at: None,
            last_password_change_at: Some(Utc::now()),
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
        self.last_password_change_at = Some(Utc::now());
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
            reason = %self.lock_reason.as_deref().unwrap_or("Unknown"),
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

    // ========================================================
    // 邮箱验证相关方法
    // ========================================================

    /// 标记邮箱已验证
    pub fn mark_email_verified(&mut self) {
        self.email_verified = true;
        self.email_verified_at = Some(Utc::now());

        tracing::info!(user_id = %self.id, email = %self.email.0, "Email verified");
    }

    /// 检查邮箱是否已验证
    pub fn is_email_verified(&self) -> bool {
        self.email_verified
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

// ============================================================
// 单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user() -> User {
        let username = Username::new("testuser").unwrap();
        let email = Email::new("test@example.com").unwrap();
        let password_hash = HashedPassword::from_hash("$2b$12$test_hash".to_string());
        let tenant_id = TenantId::new();

        User::new(username, email, password_hash, tenant_id)
    }

    #[test]
    fn test_create_user() {
        let user = create_test_user();

        assert_eq!(user.status, UserStatus::PendingVerification);
        assert_eq!(user.language, "zh-CN");
        assert_eq!(user.timezone, "Asia/Shanghai");
        assert!(!user.two_factor_enabled);
        assert_eq!(user.failed_login_count, 0);
        assert!(!user.email_verified);
        assert!(!user.phone_verified);
    }

    #[test]
    fn test_activate_user() {
        let mut user = create_test_user();
        user.activate();

        assert_eq!(user.status, UserStatus::Active);
        assert!(user.is_active());
    }

    #[test]
    fn test_deactivate_user() {
        let mut user = create_test_user();
        user.activate();
        user.deactivate();

        assert_eq!(user.status, UserStatus::Inactive);
        assert!(!user.is_active());
    }

    #[test]
    fn test_lock_user() {
        let mut user = create_test_user();
        user.lock();

        assert_eq!(user.status, UserStatus::Locked);
    }

    #[test]
    fn test_record_login() {
        let mut user = create_test_user();
        assert!(user.last_login_at.is_none());

        user.record_login();

        assert!(user.last_login_at.is_some());
    }

    #[test]
    fn test_enable_2fa() {
        let mut user = create_test_user();
        let secret = "JBSWY3DPEHPK3PXP".to_string();

        user.enable_2fa(secret.clone());

        assert!(user.two_factor_enabled);
        assert_eq!(user.two_factor_secret, Some(secret));
    }

    #[test]
    fn test_disable_2fa() {
        let mut user = create_test_user();
        user.enable_2fa("JBSWY3DPEHPK3PXP".to_string());
        user.disable_2fa();

        assert!(!user.two_factor_enabled);
        assert!(user.two_factor_secret.is_none());
    }

    #[test]
    fn test_update_password() {
        let mut user = create_test_user();
        let old_password_change_time = user.last_password_change_at;

        // 等待一小段时间确保时间戳不同
        std::thread::sleep(std::time::Duration::from_millis(10));

        let new_password_hash = HashedPassword::from_hash("$2b$12$new_hash".to_string());
        user.update_password(new_password_hash);

        assert_ne!(user.password_hash.0, "$2b$12$test_hash");
        assert!(user.last_password_change_at >= old_password_change_time);
    }

    #[test]
    fn test_add_role() {
        let mut user = create_test_user();
        let role_id = "admin".to_string();

        user.add_role(role_id.clone());

        assert_eq!(user.role_ids.len(), 1);
        assert!(user.role_ids.contains(&role_id));
    }

    #[test]
    fn test_add_duplicate_role() {
        let mut user = create_test_user();
        let role_id = "admin".to_string();

        user.add_role(role_id.clone());
        user.add_role(role_id.clone());

        assert_eq!(user.role_ids.len(), 1);
    }

    #[test]
    fn test_remove_role() {
        let mut user = create_test_user();
        let role_id = "admin".to_string();

        user.add_role(role_id.clone());
        user.remove_role(&role_id);

        assert_eq!(user.role_ids.len(), 0);
    }

    #[test]
    fn test_record_login_failure() {
        let mut user = create_test_user();

        user.record_login_failure();

        assert_eq!(user.failed_login_count, 1);
        assert!(user.last_failed_login_at.is_some());
    }

    #[test]
    fn test_auto_lock_after_10_failures() {
        let mut user = create_test_user();

        for _ in 0..10 {
            user.record_login_failure();
        }

        assert_eq!(user.status, UserStatus::Locked);
        assert!(user.locked_until.is_some());
        assert!(user.lock_reason.is_some());
        assert!(user.is_locked());
    }

    #[test]
    fn test_clear_login_failures() {
        let mut user = create_test_user();

        user.record_login_failure();
        user.record_login_failure();
        user.clear_login_failures();

        assert_eq!(user.failed_login_count, 0);
        assert!(user.last_failed_login_at.is_none());
    }

    #[test]
    fn test_lock_account() {
        let mut user = create_test_user();
        let reason = "Suspicious activity".to_string();

        user.lock_account(30, reason.clone());

        assert_eq!(user.status, UserStatus::Locked);
        assert!(user.locked_until.is_some());
        assert_eq!(user.lock_reason, Some(reason));
        assert!(user.is_locked());
    }

    #[test]
    fn test_unlock_account() {
        let mut user = create_test_user();
        user.lock_account(30, "Test lock".to_string());
        user.unlock_account();

        assert_eq!(user.status, UserStatus::Active);
        assert!(user.locked_until.is_none());
        assert!(user.lock_reason.is_none());
        assert_eq!(user.failed_login_count, 0);
        assert!(!user.is_locked());
    }

    #[test]
    fn test_should_auto_unlock() {
        let mut user = create_test_user();

        // 锁定 -1 分钟（已过期）
        user.locked_until = Some(Utc::now() - chrono::Duration::minutes(1));

        assert!(user.should_auto_unlock());
        assert!(!user.is_locked());
    }

    #[test]
    fn test_get_lock_remaining_seconds() {
        let mut user = create_test_user();

        user.lock_account(5, "Test".to_string());

        let remaining = user.get_lock_remaining_seconds();
        assert!(remaining.is_some());
        assert!(remaining.unwrap() > 0);
        assert!(remaining.unwrap() <= 300); // 5 minutes = 300 seconds
    }

    #[test]
    fn test_mark_email_verified() {
        let mut user = create_test_user();

        user.mark_email_verified();

        assert!(user.email_verified);
        assert!(user.email_verified_at.is_some());
        assert!(user.is_email_verified());
    }

    #[test]
    fn test_is_email_verified() {
        let user = create_test_user();

        assert!(!user.is_email_verified());
    }
}
