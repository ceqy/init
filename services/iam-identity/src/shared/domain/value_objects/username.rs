//! Username 值对象

use serde::{Deserialize, Serialize};
use std::fmt;

/// Username 值对象
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(pub String);

impl Username {
    /// 创建新的 Username
    pub fn new(username: impl Into<String>) -> Result<Self, UsernameError> {
        let username = username.into();
        
        // 验证用户名格式
        Self::validate(&username)?;
        
        Ok(Self(username))
    }
    
    /// 验证用户名格式
    fn validate(username: &str) -> Result<(), UsernameError> {
        // 长度检查
        if username.len() < 3 {
            return Err(UsernameError::TooShort);
        }
        
        if username.len() > 32 {
            return Err(UsernameError::TooLong);
        }
        
        // 只允许字母、数字、下划线、连字符
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(UsernameError::InvalidCharacters);
        }
        
        // 必须以字母或数字开头
        if let Some(first_char) = username.chars().next() {
            if !first_char.is_alphanumeric() {
                return Err(UsernameError::InvalidStart);
            }
        }
        
        Ok(())
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Username 错误
#[derive(Debug, thiserror::Error)]
pub enum UsernameError {
    #[error("Username is too short (minimum 3 characters)")]
    TooShort,
    
    #[error("Username is too long (maximum 32 characters)")]
    TooLong,
    
    #[error("Username contains invalid characters (only alphanumeric, underscore, and hyphen allowed)")]
    InvalidCharacters,
    
    #[error("Username must start with an alphanumeric character")]
    InvalidStart,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_username() {
        let username = Username::new("john_doe");
        assert!(username.is_ok());
        assert_eq!(username.unwrap().0, "john_doe");
    }

    #[test]
    fn test_valid_username_with_numbers() {
        let username = Username::new("user123");
        assert!(username.is_ok());
    }

    #[test]
    fn test_valid_username_with_hyphen() {
        let username = Username::new("john-doe");
        assert!(username.is_ok());
    }

    #[test]
    fn test_username_too_short() {
        let username = Username::new("ab");
        assert!(matches!(username, Err(UsernameError::TooShort)));
    }

    #[test]
    fn test_username_too_long() {
        let username = Username::new("a".repeat(33));
        assert!(matches!(username, Err(UsernameError::TooLong)));
    }

    #[test]
    fn test_username_invalid_characters() {
        let username = Username::new("john@doe");
        assert!(matches!(username, Err(UsernameError::InvalidCharacters)));
    }

    #[test]
    fn test_username_invalid_start_underscore() {
        let username = Username::new("_johndoe");
        assert!(matches!(username, Err(UsernameError::InvalidStart)));
    }

    #[test]
    fn test_username_invalid_start_hyphen() {
        let username = Username::new("-johndoe");
        assert!(matches!(username, Err(UsernameError::InvalidStart)));
    }

    #[test]
    fn test_username_with_spaces() {
        let username = Username::new("john doe");
        assert!(matches!(username, Err(UsernameError::InvalidCharacters)));
    }

    #[test]
    fn test_username_equality() {
        let username1 = Username::new("johndoe").unwrap();
        let username2 = Username::new("johndoe").unwrap();
        assert_eq!(username1, username2);
    }

    #[test]
    fn test_username_display() {
        let username = Username::new("johndoe").unwrap();
        assert_eq!(format!("{}", username), "johndoe");
    }

    #[test]
    fn test_username_minimum_length() {
        let username = Username::new("abc");
        assert!(username.is_ok());
    }

    #[test]
    fn test_username_maximum_length() {
        let username = Username::new("a".repeat(32));
        assert!(username.is_ok());
    }
}
