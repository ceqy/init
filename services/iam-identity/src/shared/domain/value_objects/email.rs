//! Email 值对象

use serde::{Deserialize, Serialize};
use std::fmt;

/// Email 值对象
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(pub String);

impl Email {
    /// 创建新的 Email
    pub fn new(email: impl Into<String>) -> Result<Self, EmailError> {
        let email = email.into();
        
        // 验证邮箱格式
        if !Self::is_valid(&email) {
            return Err(EmailError::InvalidFormat(email));
        }
        
        Ok(Self(email.to_lowercase()))
    }
    
    /// 验证邮箱格式
    fn is_valid(email: &str) -> bool {
        // 简单的邮箱格式验证
        email.contains('@') 
            && email.len() >= 3 
            && email.len() <= 254
            && !email.starts_with('@')
            && !email.ends_with('@')
    }
    
    /// 获取邮箱域名
    pub fn domain(&self) -> Option<&str> {
        self.0.split('@').nth(1)
    }
    
    /// 获取邮箱本地部分
    pub fn local_part(&self) -> Option<&str> {
        self.0.split('@').next()
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Email 错误
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Invalid email format: {0}")]
    InvalidFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let email = Email::new("test@example.com");
        assert!(email.is_ok());
        assert_eq!(email.unwrap().0, "test@example.com");
    }

    #[test]
    fn test_email_case_insensitive() {
        let email = Email::new("Test@Example.COM");
        assert!(email.is_ok());
        assert_eq!(email.unwrap().0, "test@example.com");
    }

    #[test]
    fn test_invalid_email_no_at() {
        let email = Email::new("invalid.email.com");
        assert!(email.is_err());
    }

    #[test]
    fn test_invalid_email_starts_with_at() {
        let email = Email::new("@example.com");
        assert!(email.is_err());
    }

    #[test]
    fn test_invalid_email_ends_with_at() {
        let email = Email::new("test@");
        assert!(email.is_err());
    }

    #[test]
    fn test_invalid_email_too_short() {
        let email = Email::new("a@");
        assert!(email.is_err());
    }

    #[test]
    fn test_email_domain() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(email.domain(), Some("example.com"));
    }

    #[test]
    fn test_email_local_part() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(email.local_part(), Some("test"));
    }

    #[test]
    fn test_email_equality() {
        let email1 = Email::new("test@example.com").unwrap();
        let email2 = Email::new("TEST@EXAMPLE.COM").unwrap();
        assert_eq!(email1, email2);
    }

    #[test]
    fn test_email_display() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(format!("{}", email), "test@example.com");
    }
}
