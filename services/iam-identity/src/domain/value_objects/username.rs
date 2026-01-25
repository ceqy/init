//! 用户名值对象

use cuba_errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(String);

impl Username {
    pub fn new(username: impl Into<String>) -> AppResult<Self> {
        let username = username.into();

        if username.len() < 3 {
            return Err(AppError::validation("Username must be at least 3 characters"));
        }

        if username.len() > 50 {
            return Err(AppError::validation("Username must be at most 50 characters"));
        }

        // 只允许字母、数字、下划线
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(AppError::validation(
                "Username can only contain letters, numbers, and underscores",
            ));
        }

        Ok(Self(username))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
