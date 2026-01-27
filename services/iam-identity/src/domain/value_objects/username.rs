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
    
    /// 获取字符串引用
    pub fn as_str(&self) -> &str {
        &self.0
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

