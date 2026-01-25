//! 领域服务单元测试

use cuba_common::{TenantId, UserId};
use iam_identity::auth::domain::entities::*;
use iam_identity::auth::domain::services::*;
use iam_identity::shared::domain::value_objects::*;

#[cfg(test)]
mod password_service_tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let result = PasswordService::hash_password("Test1234!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_hash_weak_password_fails() {
        let result = PasswordService::hash_password("weak");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_correct() {
        let hashed = PasswordService::hash_password("Test1234!").unwrap();
        let result = PasswordService::verify_password("Test1234!", &hashed);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let hashed = PasswordService::hash_password("Test1234!").unwrap();
        let result = PasswordService::verify_password("WrongPass!", &hashed);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}

#[cfg(test)]
mod totp_service_tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let service = TotpService::new("TestApp".to_string());
        let secret = service.generate_secret().unwrap();
        
        assert!(!secret.is_empty());
        assert!(secret.len() >= 16);
    }

    #[test]
    fn test_generate_qr_code_url() {
        let service = TotpService::new("TestApp".to_string());
        let secret = service.generate_secret().unwrap();
        let url = service.generate_qr_code_url("testuser", &secret).unwrap();
        
        assert!(url.starts_with("otpauth://totp/"));
        assert!(url.contains("TestApp"));
        assert!(url.contains("testuser"));
        assert!(url.contains(&secret));
    }

    #[test]
    fn test_verify_code_valid() {
        let service = TotpService::new("TestApp".to_string());
        let secret = service.generate_secret().unwrap();
        
        // 生成当前的 TOTP 码
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            totp_rs::Secret::Encoded(secret.clone()).to_bytes().unwrap(),
        ).unwrap();
        let code = totp.generate_current().unwrap();
        
        // 验证应该成功
        assert!(service.verify_code("testuser", &secret, &code).unwrap());
    }

    #[test]
    fn test_verify_code_invalid() {
        let service = TotpService::new("TestApp".to_string());
        let secret = service.generate_secret().unwrap();
        
        // 错误的码应该失败
        assert!(!service.verify_code("testuser", &secret, "000000").unwrap());
    }
}

#[cfg(test)]
mod backup_code_service_tests {
    use super::*;

    #[test]
    fn test_generate_backup_codes() {
        let codes = BackupCodeService::generate_backup_codes(8);
        
        assert_eq!(codes.len(), 8);
        
        // 每个码应该是 8 个字符
        for code in &codes {
            assert_eq!(code.len(), 8);
            // 应该只包含字母和数字
            assert!(code.chars().all(|c| c.is_alphanumeric()));
        }
        
        // 所有码应该是唯一的
        let unique_codes: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique_codes.len(), codes.len());
    }

    #[test]
    fn test_generate_default_count() {
        let codes = BackupCodeService::generate_backup_codes(10);
        assert_eq!(codes.len(), 10);
    }

    #[test]
    fn test_backup_codes_randomness() {
        let codes1 = BackupCodeService::generate_backup_codes(5);
        let codes2 = BackupCodeService::generate_backup_codes(5);
        
        // 两次生成的码应该不同
        assert_ne!(codes1, codes2);
    }
}

#[cfg(test)]
mod login_attempt_service_tests {
    use super::*;

    #[test]
    fn test_should_require_captcha() {
        let service = LoginAttemptService::new(5, 15);
        
        // 3 次失败后应该要求验证码
        assert!(!service.should_require_captcha("user1", 0));
        assert!(!service.should_require_captcha("user1", 1));
        assert!(!service.should_require_captcha("user1", 2));
        assert!(service.should_require_captcha("user1", 3));
        assert!(service.should_require_captcha("user1", 4));
    }

    #[test]
    fn test_should_lock_account() {
        let service = LoginAttemptService::new(5, 15);
        
        // 5 次失败后应该锁定账户
        assert!(!service.should_lock_account("user1", 0));
        assert!(!service.should_lock_account("user1", 4));
        assert!(service.should_lock_account("user1", 5));
        assert!(service.should_lock_account("user1", 10));
    }

    #[test]
    fn test_get_lock_duration() {
        let service = LoginAttemptService::new(5, 15);
        
        assert_eq!(service.get_lock_duration(), 15);
    }
}

#[cfg(test)]
mod suspicious_login_detector_tests {
    use super::*;

    fn create_test_login_log(ip: &str, user_agent: &str) -> LoginLog {
        LoginLog::new(
            UserId::new(),
            TenantId::new(),
            ip.to_string(),
            user_agent.to_string(),
            true,
        )
    }

    #[test]
    fn test_detect_new_location() {
        let detector = SuspiciousLoginDetector::new();
        
        let previous_logs = vec![
            create_test_login_log("192.168.1.1", "Mozilla/5.0"),
        ];
        
        let current_log = create_test_login_log("10.0.0.1", "Mozilla/5.0");
        
        let result = detector.detect(&current_log, &previous_logs);
        assert!(result.is_new_location);
    }

    #[test]
    fn test_detect_new_device() {
        let detector = SuspiciousLoginDetector::new();
        
        let previous_logs = vec![
            create_test_login_log("192.168.1.1", "Mozilla/5.0 (Windows)"),
        ];
        
        let current_log = create_test_login_log("192.168.1.1", "Mozilla/5.0 (iPhone)");
        
        let result = detector.detect(&current_log, &previous_logs);
        assert!(result.is_new_device);
    }

    #[test]
    fn test_detect_unusual_time() {
        let detector = SuspiciousLoginDetector::new();
        
        // 凌晨 3 点登录应该被标记为异常时间
        let current_log = create_test_login_log("192.168.1.1", "Mozilla/5.0");
        
        let result = detector.detect(&current_log, &[]);
        // 注意：这个测试依赖于当前时间，可能需要 mock
    }

    #[test]
    fn test_no_suspicious_activity() {
        let detector = SuspiciousLoginDetector::new();
        
        let previous_logs = vec![
            create_test_login_log("192.168.1.1", "Mozilla/5.0 (Windows)"),
        ];
        
        let current_log = create_test_login_log("192.168.1.1", "Mozilla/5.0 (Windows)");
        
        let result = detector.detect(&current_log, &previous_logs);
        assert!(!result.is_new_location);
        assert!(!result.is_new_device);
    }
}
