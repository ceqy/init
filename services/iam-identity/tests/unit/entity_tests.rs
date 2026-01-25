//! 实体单元测试

use chrono::Utc;
use cuba_common::{TenantId, UserId};
use iam_identity::shared::domain::entities::*;
use iam_identity::shared::domain::value_objects::*;

#[cfg(test)]
mod user_tests {
    use super::*;

    fn create_test_user() -> User {
        User::new(
            Username::new("testuser").unwrap(),
            Email::new("test@example.com").unwrap(),
            HashedPassword::from_plain("Test1234!").unwrap(),
            TenantId::new(),
        )
    }

    #[test]
    fn test_create_user() {
        let user = create_test_user();
        assert_eq!(user.username.0, "testuser");
        assert_eq!(user.email.0, "test@example.com");
        assert_eq!(user.status, UserStatus::PendingVerification);
        assert!(!user.two_factor_enabled);
        assert!(!user.email_verified);
    }

    #[test]
    fn test_user_activate() {
        let mut user = create_test_user();
        user.activate();
        assert_eq!(user.status, UserStatus::Active);
        assert!(user.is_active());
    }

    #[test]
    fn test_user_deactivate() {
        let mut user = create_test_user();
        user.activate();
        user.deactivate();
        assert_eq!(user.status, UserStatus::Inactive);
        assert!(!user.is_active());
    }

    #[test]
    fn test_user_lock() {
        let mut user = create_test_user();
        user.lock();
        assert_eq!(user.status, UserStatus::Locked);
    }

    #[test]
    fn test_user_record_login() {
        let mut user = create_test_user();
        assert!(user.last_login_at.is_none());
        
        user.record_login();
        assert!(user.last_login_at.is_some());
    }

    #[test]
    fn test_user_enable_2fa() {
        let mut user = create_test_user();
        assert!(!user.two_factor_enabled);
        
        user.enable_2fa("test_secret".to_string());
        assert!(user.two_factor_enabled);
        assert_eq!(user.two_factor_secret, Some("test_secret".to_string()));
    }

    #[test]
    fn test_user_disable_2fa() {
        let mut user = create_test_user();
        user.enable_2fa("test_secret".to_string());
        
        user.disable_2fa();
        assert!(!user.two_factor_enabled);
        assert!(user.two_factor_secret.is_none());
    }

    #[test]
    fn test_user_update_password() {
        let mut user = create_test_user();
        let old_hash = user.password_hash.clone();
        
        let new_hash = HashedPassword::from_plain("NewPass123!").unwrap();
        user.update_password(new_hash.clone());
        
        assert_ne!(user.password_hash.0, old_hash.0);
        assert_eq!(user.password_hash.0, new_hash.0);
    }

    #[test]
    fn test_user_add_role() {
        let mut user = create_test_user();
        assert!(user.role_ids.is_empty());
        
        user.add_role("admin".to_string());
        assert_eq!(user.role_ids.len(), 1);
        assert!(user.role_ids.contains(&"admin".to_string()));
        
        // 添加重复角色不应该增加
        user.add_role("admin".to_string());
        assert_eq!(user.role_ids.len(), 1);
    }

    #[test]
    fn test_user_remove_role() {
        let mut user = create_test_user();
        user.add_role("admin".to_string());
        user.add_role("user".to_string());
        assert_eq!(user.role_ids.len(), 2);
        
        user.remove_role("admin");
        assert_eq!(user.role_ids.len(), 1);
        assert!(!user.role_ids.contains(&"admin".to_string()));
        assert!(user.role_ids.contains(&"user".to_string()));
    }

    #[test]
    fn test_user_record_login_failure() {
        let mut user = create_test_user();
        assert_eq!(user.failed_login_count, 0);
        
        user.record_login_failure();
        assert_eq!(user.failed_login_count, 1);
        assert!(user.last_failed_login_at.is_some());
    }

    #[test]
    fn test_user_auto_lock_after_10_failures() {
        let mut user = create_test_user();
        
        // 记录 10 次失败
        for _ in 0..10 {
            user.record_login_failure();
        }
        
        // 应该自动锁定
        assert_eq!(user.status, UserStatus::Locked);
        assert!(user.locked_until.is_some());
        assert!(user.is_locked());
    }

    #[test]
    fn test_user_clear_login_failures() {
        let mut user = create_test_user();
        user.record_login_failure();
        user.record_login_failure();
        assert_eq!(user.failed_login_count, 2);
        
        user.clear_login_failures();
        assert_eq!(user.failed_login_count, 0);
        assert!(user.last_failed_login_at.is_none());
    }

    #[test]
    fn test_user_lock_account() {
        let mut user = create_test_user();
        user.lock_account(30, "Test lock".to_string());
        
        assert_eq!(user.status, UserStatus::Locked);
        assert!(user.locked_until.is_some());
        assert_eq!(user.lock_reason, Some("Test lock".to_string()));
        assert!(user.is_locked());
    }

    #[test]
    fn test_user_unlock_account() {
        let mut user = create_test_user();
        user.lock_account(30, "Test lock".to_string());
        
        user.unlock_account();
        assert!(user.locked_until.is_none());
        assert!(user.lock_reason.is_none());
        assert_eq!(user.failed_login_count, 0);
        assert!(!user.is_locked());
    }

    #[test]
    fn test_user_mark_email_verified() {
        let mut user = create_test_user();
        assert!(!user.email_verified);
        assert!(user.email_verified_at.is_none());
        
        user.mark_email_verified();
        assert!(user.email_verified);
        assert!(user.email_verified_at.is_some());
        assert!(user.is_email_verified());
    }
}

#[cfg(test)]
mod tenant_tests {
    use super::*;
    use iam_identity::shared::domain::entities::Tenant;

    fn create_test_tenant() -> Tenant {
        Tenant::new(
            "Test Tenant".to_string(),
            "test-tenant".to_string(),
        )
    }

    #[test]
    fn test_create_tenant() {
        let tenant = create_test_tenant();
        assert_eq!(tenant.name, "Test Tenant");
        assert_eq!(tenant.slug, "test-tenant");
        assert!(tenant.is_active());
    }

    #[test]
    fn test_tenant_activate() {
        let mut tenant = create_test_tenant();
        tenant.deactivate();
        assert!(!tenant.is_active());
        
        tenant.activate();
        assert!(tenant.is_active());
    }

    #[test]
    fn test_tenant_deactivate() {
        let mut tenant = create_test_tenant();
        tenant.deactivate();
        assert!(!tenant.is_active());
    }

    #[test]
    fn test_tenant_suspend() {
        let mut tenant = create_test_tenant();
        tenant.suspend("Payment overdue".to_string());
        assert!(tenant.is_suspended());
    }

    #[test]
    fn test_tenant_update_settings() {
        let mut tenant = create_test_tenant();
        let mut settings = TenantSettings::default();
        settings.require_2fa = true;
        
        tenant.update_settings(settings.clone());
        assert!(tenant.settings.require_2fa);
    }
}

#[cfg(test)]
mod email_verification_tests {
    use super::*;
    use iam_identity::shared::domain::entities::EmailVerification;

    #[test]
    fn test_create_email_verification() {
        let verification = EmailVerification::new(
            UserId::new(),
            TenantId::new(),
            Email::new("test@example.com").unwrap(),
        );
        
        assert_eq!(verification.code.len(), 6);
        assert!(!verification.is_expired());
        assert!(!verification.is_verified);
    }

    #[test]
    fn test_email_verification_verify_correct_code() {
        let verification = EmailVerification::new(
            UserId::new(),
            TenantId::new(),
            Email::new("test@example.com").unwrap(),
        );
        
        let code = verification.code.clone();
        let result = verification.verify(&code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_email_verification_verify_incorrect_code() {
        let verification = EmailVerification::new(
            UserId::new(),
            TenantId::new(),
            Email::new("test@example.com").unwrap(),
        );
        
        let result = verification.verify("000000");
        assert!(result.is_err());
    }

    #[test]
    fn test_email_verification_code_format() {
        let verification = EmailVerification::new(
            UserId::new(),
            TenantId::new(),
            Email::new("test@example.com").unwrap(),
        );
        
        // 验证码应该是 6 位数字
        assert_eq!(verification.code.len(), 6);
        assert!(verification.code.chars().all(|c| c.is_numeric()));
    }
}

#[cfg(test)]
mod phone_verification_tests {
    use super::*;
    use iam_identity::shared::domain::entities::PhoneVerification;

    #[test]
    fn test_create_phone_verification() {
        let verification = PhoneVerification::new(
            UserId::new(),
            TenantId::new(),
            "+1234567890".to_string(),
        );
        
        assert_eq!(verification.code.len(), 6);
        assert!(!verification.is_expired());
        assert!(!verification.is_verified);
    }

    #[test]
    fn test_phone_verification_verify_correct_code() {
        let verification = PhoneVerification::new(
            UserId::new(),
            TenantId::new(),
            "+1234567890".to_string(),
        );
        
        let code = verification.code.clone();
        let result = verification.verify(&code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_phone_verification_verify_incorrect_code() {
        let verification = PhoneVerification::new(
            UserId::new(),
            TenantId::new(),
            "+1234567890".to_string(),
        );
        
        let result = verification.verify("000000");
        assert!(result.is_err());
    }
}
