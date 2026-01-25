//! 值对象单元测试

use iam_identity::shared::domain::value_objects::*;

#[cfg(test)]
mod email_tests {
    use super::*;

    #[test]
    fn test_email_comprehensive() {
        // 有效的邮箱
        assert!(Email::new("user@example.com").is_ok());
        assert!(Email::new("user.name@example.com").is_ok());
        assert!(Email::new("user+tag@example.com").is_ok());
        assert!(Email::new("user123@example.co.uk").is_ok());
        
        // 无效的邮箱
        assert!(Email::new("").is_err());
        assert!(Email::new("@example.com").is_err());
        assert!(Email::new("user@").is_err());
        assert!(Email::new("user").is_err());
        assert!(Email::new("user@.com").is_err());
    }

    #[test]
    fn test_email_normalization() {
        let email1 = Email::new("User@Example.COM").unwrap();
        let email2 = Email::new("user@example.com").unwrap();
        assert_eq!(email1, email2);
    }
}

#[cfg(test)]
mod username_tests {
    use super::*;

    #[test]
    fn test_username_comprehensive() {
        // 有效的用户名
        assert!(Username::new("john").is_ok());
        assert!(Username::new("john_doe").is_ok());
        assert!(Username::new("john-doe").is_ok());
        assert!(Username::new("john123").is_ok());
        assert!(Username::new("123john").is_ok());
        
        // 无效的用户名
        assert!(Username::new("ab").is_err()); // 太短
        assert!(Username::new("a".repeat(33)).is_err()); // 太长
        assert!(Username::new("_john").is_err()); // 以下划线开头
        assert!(Username::new("-john").is_err()); // 以连字符开头
        assert!(Username::new("john@doe").is_err()); // 包含特殊字符
        assert!(Username::new("john doe").is_err()); // 包含空格
    }

    #[test]
    fn test_username_boundaries() {
        // 最小长度
        assert!(Username::new("abc").is_ok());
        assert!(Username::new("ab").is_err());
        
        // 最大长度
        assert!(Username::new("a".repeat(32)).is_ok());
        assert!(Username::new("a".repeat(33)).is_err());
    }
}

#[cfg(test)]
mod password_tests {
    use super::*;

    #[test]
    fn test_password_strength_validation() {
        // 强密码
        assert!(Password::validate("Test1234!").is_ok());
        assert!(Password::validate("MyP@ssw0rd").is_ok());
        assert!(Password::validate("Secure#Pass123").is_ok());
        
        // 弱密码
        assert!(Password::validate("test").is_err()); // 太短
        assert!(Password::validate("testtest").is_err()); // 没有大写和数字
        assert!(Password::validate("TESTTEST").is_err()); // 没有小写和数字
        assert!(Password::validate("Test1234").is_err()); // 没有特殊字符（只有3种类型）
    }

    #[test]
    fn test_password_hashing_and_verification() {
        let plain = "Test1234!";
        let hashed = HashedPassword::from_plain(plain).unwrap();
        
        // 正确的密码应该验证成功
        assert!(hashed.verify(plain).unwrap());
        
        // 错误的密码应该验证失败
        assert!(!hashed.verify("WrongPassword!").unwrap());
    }

    #[test]
    fn test_password_hash_uniqueness() {
        let plain = "Test1234!";
        let hash1 = HashedPassword::from_plain(plain).unwrap();
        let hash2 = HashedPassword::from_plain(plain).unwrap();
        
        // 即使密码相同，哈希也应该不同（因为盐不同）
        assert_ne!(hash1.0, hash2.0);
    }
}

#[cfg(test)]
mod tenant_context_tests {
    use super::*;
    use cuba_common::TenantId;

    #[test]
    fn test_tenant_context_creation() {
        let tenant_id = TenantId::new();
        let settings = TenantSettings::default();
        let context = TenantContext::new(
            tenant_id.clone(),
            "Test Tenant".to_string(),
            settings,
        );
        
        assert_eq!(context.tenant_id, tenant_id);
        assert_eq!(context.tenant_name, "Test Tenant");
    }

    #[test]
    fn test_password_policy_validation() {
        let policy = PasswordPolicy {
            min_length: 10,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special_char: true,
            max_age_days: Some(90),
        };
        
        // 符合策略的密码
        assert!(policy.validate("Test1234!@").is_ok());
        
        // 不符合策略的密码
        assert!(policy.validate("Test123!").is_err()); // 太短
        assert!(policy.validate("test1234!@").is_err()); // 没有大写
        assert!(policy.validate("TEST1234!@").is_err()); // 没有小写
        assert!(policy.validate("TestTest!@").is_err()); // 没有数字
        assert!(policy.validate("Test1234ab").is_err()); // 没有特殊字符
    }

    #[test]
    fn test_tenant_settings_2fa_requirement() {
        let mut settings = TenantSettings::default();
        settings.require_2fa = true;
        
        let context = TenantContext::new(
            TenantId::new(),
            "Test".to_string(),
            settings,
        );
        
        assert!(context.requires_2fa());
    }

    #[test]
    fn test_tenant_settings_user_limit() {
        let mut settings = TenantSettings::default();
        settings.max_users = Some(100);
        
        let context = TenantContext::new(
            TenantId::new(),
            "Test".to_string(),
            settings,
        );
        
        assert!(!context.is_user_limit_reached(99));
        assert!(context.is_user_limit_reached(100));
        assert!(context.is_user_limit_reached(101));
    }
}
