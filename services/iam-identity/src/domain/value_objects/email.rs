//! 邮箱值对象

use cuba_errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(email: impl Into<String>) -> AppResult<Self> {
        let email = email.into();

        // 简单的邮箱验证
        if !email.contains('@') || !email.contains('.') {
            return Err(AppError::validation("Invalid email format"));
        }

        if email.len() > 255 {
            return Err(AppError::validation("Email too long"));
        }

        Ok(Self(email.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
