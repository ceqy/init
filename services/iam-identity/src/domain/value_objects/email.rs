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

        // 使用 email_address crate 进行严格的 RFC 5322 验证
        if !email_address::EmailAddress::is_valid(&email) {
            return Err(EmailError::InvalidFormat(email));
        }

        Ok(Self(email.to_lowercase()))
    }

    /// 获取邮箱域名
    pub fn domain(&self) -> Option<&str> {
        self.0.split('@').nth(1)
    }

    /// 获取邮箱本地部分
    pub fn local_part(&self) -> Option<&str> {
        self.0.split('@').next()
    }

    /// 获取字符串引用
    pub fn as_str(&self) -> &str {
        &self.0
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
    fn test_valid_emails() {
        assert!(Email::new("user@example.com").is_ok());
        assert!(Email::new("user.name@example.com").is_ok());
        assert!(Email::new("user+tag@example.co.uk").is_ok());
        assert!(Email::new("user_name@example-domain.com").is_ok());
    }

    #[test]
    fn test_invalid_emails() {
        // 太短
        assert!(Email::new("a@b").is_err());

        // 多个 @
        assert!(Email::new("user@@example.com").is_err());
        assert!(Email::new("@@@@").is_err());

        // 缺少 @
        assert!(Email::new("userexample.com").is_err());

        // @ 在开头或结尾
        assert!(Email::new("@example.com").is_err());
        assert!(Email::new("user@").is_err());

        // 无效字符
        assert!(Email::new("user name@example.com").is_err());

        // 空字符串
        assert!(Email::new("").is_err());
    }

    #[test]
    fn test_email_lowercase() {
        let email = Email::new("User@Example.COM").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn test_email_domain() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(email.domain(), Some("example.com"));
    }

    #[test]
    fn test_email_local_part() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(email.local_part(), Some("user"));
    }
}