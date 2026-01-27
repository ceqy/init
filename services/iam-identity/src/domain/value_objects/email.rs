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

