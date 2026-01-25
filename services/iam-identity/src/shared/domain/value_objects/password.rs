//! 密码值对象

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use cuba_errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};

/// 哈希后的密码
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashedPassword(String);

impl HashedPassword {
    /// 从明文密码创建哈希密码
    pub fn from_plain(password: &str) -> AppResult<Self> {
        // 验证密码强度
        validate_password_strength(password)?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::internal(format!("Failed to hash password: {}", e)))?;

        Ok(Self(hash.to_string()))
    }

    /// 从已有的哈希值创建
    pub fn from_hash(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// 验证密码
    pub fn verify(&self, password: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(&self.0)
            .map_err(|e| AppError::internal(format!("Invalid password hash: {}", e)))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 验证密码强度
fn validate_password_strength(password: &str) -> AppResult<()> {
    if password.len() < 8 {
        return Err(AppError::validation("Password must be at least 8 characters"));
    }

    if password.len() > 128 {
        return Err(AppError::validation("Password must be at most 128 characters"));
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    if !has_uppercase || !has_lowercase || !has_digit {
        return Err(AppError::validation(
            "Password must contain uppercase, lowercase, and digit",
        ));
    }

    Ok(())
}
