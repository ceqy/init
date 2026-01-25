//! 认证流程集成测试

use chrono::Utc;
use cuba_common::{TenantId, UserId};
use iam_identity::auth::domain::entities::*;
use iam_identity::auth::domain::services::*;
use iam_identity::shared::domain::entities::*;
use iam_identity::shared::domain::value_objects::*;

#[cfg(test)]
mod login_flow_tests {
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
    fn test_successful_login_flow() {
        // 1. 创建用户
        let mut user = create_test_user();
        user.activate();
        
        // 2. 验证密码
        let password_valid = user.password_hash.verify("Test1234!").unwrap();
        assert!(password_valid);
        
        // 3. 记录登录
        user.record_login();
        assert!(user.last_login_at.is_some());
        
        // 4. 清除失败记录
        user.clear_login_failures();
        assert_eq!(user.failed_login_count, 0);
    }

    #[test]
    fn test_failed_login_flow() {
        let mut user = create_test_user();
        user.activate();
        
        // 1. 错误的密码
        let password_valid = user.password_hash.verify("WrongPassword!").unwrap();
        assert!(!password_valid);
        
        // 2. 记录失败
        user.record_login_failure();
        assert_eq!(user.failed_login_count, 1);
        assert!(user.last_failed_login_at.is_some());
    }

    #[test]
    fn test_account_lockout_flow() {
        let mut user = create_test_user();
        user.activate();
        
        // 1. 记录 10 次失败
        for _ in 0..10 {
            user.record_login_failure();
        }
        
        // 2. 账户应该被锁定
        assert_eq!(user.status, UserStatus::Locked);
        assert!(user.is_locked());
        assert!(user.locked_until.is_some());
        
        // 3. 解锁账户
        user.unlock_account();
        assert!(!user.is_locked());
        assert_eq!(user.failed_login_count, 0);
    }

    #[test]
    fn test_login_with_2fa_flow() {
        let mut user = create_test_user();
        user.activate();
        
        // 1. 启用 2FA
        let totp_service = TotpService::new("TestApp".to_string());
        let secret = totp_service.generate_secret().unwrap();
        user.enable_2fa(secret.clone());
        
        assert!(user.two_factor_enabled);
        
        // 2. 验证密码
        let password_valid = user.password_hash.verify("Test1234!").unwrap();
        assert!(password_valid);
        
        // 3. 生成并验证 TOTP 码
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            totp_rs::Secret::Encoded(secret.clone()).to_bytes().unwrap(),
        ).unwrap();
        let code = totp.generate_current().unwrap();
        
        let code_valid = totp_service.verify_code("testuser", &secret, &code).unwrap();
        assert!(code_valid);
        
        // 4. 记录登录
        user.record_login();
        assert!(user.last_login_at.is_some());
    }

    #[test]
    fn test_captcha_requirement_flow() {
        let login_attempt_service = LoginAttemptService::new(5, 15);
        
        // 1. 前 2 次失败不需要验证码
        assert!(!login_attempt_service.should_require_captcha("user1", 0));
        assert!(!login_attempt_service.should_require_captcha("user1", 2));
        
        // 2. 第 3 次失败需要验证码
        assert!(login_attempt_service.should_require_captcha("user1", 3));
        
        // 3. 第 5 次失败需要锁定账户
        assert!(login_attempt_service.should_lock_account("user1", 5));
    }
}

#[cfg(test)]
mod password_reset_flow_tests {
    use super::*;
    use iam_identity::auth::domain::entities::PasswordResetToken;

    #[test]
    fn test_password_reset_flow() {
        let mut user = create_test_user();
        let old_password_hash = user.password_hash.clone();
        
        // 1. 创建密码重置令牌
        let token = PasswordResetToken::new(
            user.id.clone(),
            user.tenant_id.clone(),
            user.email.clone(),
        );
        
        assert!(!token.is_expired());
        assert!(!token.is_used);
        
        // 2. 验证令牌
        let token_hash = token.token_hash.clone();
        let verification_result = token.verify(&token_hash);
        assert!(verification_result.is_ok());
        
        // 3. 更新密码
        let new_password = HashedPassword::from_plain("NewPass123!").unwrap();
        user.update_password(new_password.clone());
        
        // 4. 验证新密码
        assert_ne!(user.password_hash.0, old_password_hash.0);
        assert!(user.password_hash.verify("NewPass123!").unwrap());
        assert!(!user.password_hash.verify("Test1234!").unwrap());
    }

    #[test]
    fn test_password_reset_token_expiration() {
        let user = create_test_user();
        let token = PasswordResetToken::new(
            user.id.clone(),
            user.tenant_id.clone(),
            user.email.clone(),
        );
        
        // 新创建的令牌不应该过期
        assert!(!token.is_expired());
        
        // 令牌应该在 1 小时后过期（需要 mock 时间来测试）
    }
}

#[cfg(test)]
mod email_verification_flow_tests {
    use super::*;

    #[test]
    fn test_email_verification_flow() {
        let mut user = create_test_user();
        assert!(!user.is_email_verified());
        
        // 1. 创建邮箱验证
        let verification = EmailVerification::new(
            user.id.clone(),
            user.tenant_id.clone(),
            user.email.clone(),
        );
        
        assert_eq!(verification.code.len(), 6);
        assert!(!verification.is_expired());
        
        // 2. 验证验证码
        let code = verification.code.clone();
        let result = verification.verify(&code);
        assert!(result.is_ok());
        
        // 3. 标记邮箱已验证
        user.mark_email_verified();
        assert!(user.is_email_verified());
        assert!(user.email_verified_at.is_some());
    }

    #[test]
    fn test_email_verification_wrong_code() {
        let user = create_test_user();
        let verification = EmailVerification::new(
            user.id.clone(),
            user.tenant_id.clone(),
            user.email.clone(),
        );
        
        // 错误的验证码应该失败
        let result = verification.verify("000000");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod webauthn_flow_tests {
    use super::*;
    use iam_identity::auth::domain::entities::WebAuthnCredential;

    #[test]
    fn test_webauthn_registration_flow() {
        let user = create_test_user();
        
        // 1. 创建 WebAuthn 凭证
        let credential = WebAuthnCredential::new(
            user.id.clone(),
            user.tenant_id.clone(),
            "credential_id".to_string(),
            vec![1, 2, 3, 4], // 公钥
            0, // 签名计数
            "Test Device".to_string(),
        );
        
        assert_eq!(credential.name, "Test Device");
        assert_eq!(credential.sign_count, 0);
        assert!(!credential.is_backup_eligible);
    }

    #[test]
    fn test_webauthn_authentication_flow() {
        let user = create_test_user();
        let mut credential = WebAuthnCredential::new(
            user.id.clone(),
            user.tenant_id.clone(),
            "credential_id".to_string(),
            vec![1, 2, 3, 4],
            0,
            "Test Device".to_string(),
        );
        
        // 1. 更新签名计数
        let old_count = credential.sign_count;
        credential.update_sign_count(1);
        assert_eq!(credential.sign_count, 1);
        assert!(credential.sign_count > old_count);
        
        // 2. 记录最后使用时间
        credential.record_usage();
        assert!(credential.last_used_at.is_some());
    }
}

#[cfg(test)]
mod backup_code_flow_tests {
    use super::*;
    use iam_identity::auth::domain::entities::BackupCode;

    #[test]
    fn test_backup_code_generation_and_usage() {
        let user = create_test_user();
        
        // 1. 生成备份码
        let codes = BackupCodeService::generate_backup_codes(8);
        assert_eq!(codes.len(), 8);
        
        // 2. 创建备份码实体
        let code_hash = HashedPassword::from_plain(&codes[0]).unwrap();
        let mut backup_code = BackupCode::new(
            user.id.clone(),
            user.tenant_id.clone(),
            code_hash,
        );
        
        assert!(!backup_code.is_used);
        
        // 3. 使用备份码
        backup_code.mark_as_used();
        assert!(backup_code.is_used);
        assert!(backup_code.used_at.is_some());
    }

    #[test]
    fn test_backup_code_uniqueness() {
        let codes1 = BackupCodeService::generate_backup_codes(5);
        let codes2 = BackupCodeService::generate_backup_codes(5);
        
        // 两次生成的码应该不同
        assert_ne!(codes1, codes2);
        
        // 每组内的码应该唯一
        let unique_codes1: std::collections::HashSet<_> = codes1.iter().collect();
        assert_eq!(unique_codes1.len(), codes1.len());
    }
}

fn create_test_user() -> User {
    User::new(
        Username::new("testuser").unwrap(),
        Email::new("test@example.com").unwrap(),
        HashedPassword::from_plain("Test1234!").unwrap(),
        TenantId::new(),
    )
}
