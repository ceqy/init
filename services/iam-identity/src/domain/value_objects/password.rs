//! Password 值对象

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 哈希后的密码
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashedPassword(pub String);

impl HashedPassword {
    /// 从明文密码创建哈希密码
    pub fn from_plain(plain_password: &str) -> Result<Self, PasswordError> {
        // 验证密码强度
        Password::validate(plain_password)?;
        
        // 使用 Argon2 哈希密码
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(plain_password.as_bytes(), &salt)
            .map_err(|e| PasswordError::HashingFailed(e.to_string()))?
            .to_string();
        
        Ok(Self(password_hash))
    }
    
    /// 验证明文密码是否匹配
    pub fn verify(&self, plain_password: &str) -> Result<bool, PasswordError> {
        let parsed_hash = PasswordHash::new(&self.0)
            .map_err(|e| PasswordError::InvalidHash(e.to_string()))?;
        
        Ok(Argon2::default()
            .verify_password(plain_password.as_bytes(), &parsed_hash)
            .is_ok())
    }
    
    /// 从已有的哈希字符串创建
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }
    
    /// 获取字符串引用
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HashedPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// 明文密码（仅用于验证）
pub struct Password(String);

impl Password {
    /// 创建新的 Password（验证后）
    pub fn new(password: impl Into<String>) -> Result<Self, PasswordError> {
        let password = password.into();
        Self::validate(&password)?;
        Ok(Self(password))
    }
    
    /// 获取字符串引用
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// 验证密码强度
    pub fn validate(password: &str) -> Result<(), PasswordError> {
        // 长度检查
        if password.len() < 8 {
            return Err(PasswordError::TooShort);
        }
        
        if password.len() > 128 {
            return Err(PasswordError::TooLong);
        }
        
        // 复杂度检查
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());
        
        let complexity_count = [has_lowercase, has_uppercase, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();
        
        if complexity_count < 3 {
            return Err(PasswordError::TooWeak);
        }
        
        Ok(())
    }
}

/// Password 错误
#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Password is too short (minimum 8 characters)")]
    TooShort,
    
    #[error("Password is too long (maximum 128 characters)")]
    TooLong,
    
    #[error("Password is too weak (must contain at least 3 of: lowercase, uppercase, digit, special character)")]
    TooWeak,
    
    #[error("Password hashing failed: {0}")]
    HashingFailed(String),
    
    #[error("Invalid password hash: {0}")]
    InvalidHash(String),
}

impl From<PasswordError> for cuba_errors::AppError {
    fn from(err: PasswordError) -> Self {
        cuba_errors::AppError::validation(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        let result = Password::validate("Test1234!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_too_short() {
        let result = Password::validate("Test1!");
        assert!(matches!(result, Err(PasswordError::TooShort)));
    }

    #[test]
    fn test_password_too_long() {
        let result = Password::validate(&"a".repeat(129));
        assert!(matches!(result, Err(PasswordError::TooLong)));
    }

    #[test]
    fn test_password_too_weak_no_uppercase() {
        let result = Password::validate("test1234!");
        assert!(matches!(result, Err(PasswordError::TooWeak)));
    }

    #[test]
    fn test_password_too_weak_no_lowercase() {
        let result = Password::validate("TEST1234!");
        assert!(matches!(result, Err(PasswordError::TooWeak)));
    }

    #[test]
    fn test_password_too_weak_no_digit() {
        let result = Password::validate("TestTest!");
        assert!(matches!(result, Err(PasswordError::TooWeak)));
    }

    #[test]
    fn test_password_too_weak_no_special() {
        let result = Password::validate("Test1234");
        assert!(matches!(result, Err(PasswordError::TooWeak)));
    }

    #[test]
    fn test_password_with_three_types() {
        // 小写 + 大写 + 数字
        let result = Password::validate("TestPassword123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_hashed_password_creation() {
        let hashed = HashedPassword::from_plain("Test1234!");
        assert!(hashed.is_ok());
    }

    #[test]
    fn test_hashed_password_verify_correct() {
        let hashed = HashedPassword::from_plain("Test1234!").unwrap();
        let result = hashed.verify("Test1234!");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_hashed_password_verify_incorrect() {
        let hashed = HashedPassword::from_plain("Test1234!").unwrap();
        let result = hashed.verify("WrongPassword!");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_hashed_password_display_redacted() {
        let hashed = HashedPassword::from_plain("Test1234!").unwrap();
        assert_eq!(format!("{}", hashed), "[REDACTED]");
    }

    #[test]
    fn test_password_minimum_length() {
        let result = Password::validate("Test123!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_maximum_length() {
        let mut password = "Test123!".to_string();
        password.push_str(&"a".repeat(120));
        let result = Password::validate(&password);
        assert!(result.is_ok());
    }
}
