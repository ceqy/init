//! Password 值对象
//!
//! 增强的密码策略，包括：
//! - 常见密码检查
//! - 密码强度评分
//! - 可配置的复杂度要求

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 常见弱密码列表（前 100 个最常用密码的子集）
const COMMON_PASSWORDS: &[&str] = &[
    "password",
    "123456",
    "12345678",
    "qwerty",
    "abc123",
    "monkey",
    "1234567",
    "letmein",
    "trustno1",
    "dragon",
    "baseball",
    "111111",
    "iloveyou",
    "master",
    "sunshine",
    "ashley",
    "bailey",
    "passw0rd",
    "shadow",
    "123123",
    "654321",
    "superman",
    "qazwsx",
    "michael",
    "football",
    "welcome",
    "jesus",
    "ninja",
    "mustang",
    "password1",
    "123456789",
    "admin",
    "root",
    "test",
    "guest",
];

/// 密码强度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    /// 弱密码（0-40 分）
    Weak,
    /// 中等密码（41-70 分）
    Medium,
    /// 强密码（71-90 分）
    Strong,
    /// 非常强（91-100 分）
    VeryStrong,
}

impl PasswordStrength {
    /// 从分数计算强度
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=40 => Self::Weak,
            41..=70 => Self::Medium,
            71..=90 => Self::Strong,
            _ => Self::VeryStrong,
        }
    }
}

/// 密码验证配置（内部使用）
#[derive(Debug, Clone)]
pub struct PasswordValidationConfig {
    /// 最小长度
    pub min_length: usize,
    /// 最大长度
    pub max_length: usize,
    /// 需要的最小复杂度类型数量（小写、大写、数字、特殊字符）
    pub min_complexity_types: usize,
    /// 是否检查常见密码
    pub check_common_passwords: bool,
    /// 最小强度要求
    pub min_strength: PasswordStrength,
}

impl Default for PasswordValidationConfig {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            min_complexity_types: 3,
            check_common_passwords: true,
            min_strength: PasswordStrength::Medium,
        }
    }
}

/// 哈希后的密码
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashedPassword(pub String);

impl HashedPassword {
    /// 从明文密码创建哈希密码（使用默认配置）
    pub fn from_plain(plain_password: &str) -> Result<Self, PasswordError> {
        Self::from_plain_with_config(plain_password, &PasswordValidationConfig::default())
    }

    /// 从明文密码创建哈希密码（使用自定义配置）
    pub fn from_plain_with_config(
        plain_password: &str,
        config: &PasswordValidationConfig,
    ) -> Result<Self, PasswordError> {
        // 验证密码强度
        Password::validate_with_config(plain_password, config)?;

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
        let parsed_hash =
            PasswordHash::new(&self.0).map_err(|e| PasswordError::InvalidHash(e.to_string()))?;

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

    /// 验证密码强度（使用默认配置）
    pub fn validate(password: &str) -> Result<(), PasswordError> {
        Self::validate_with_config(password, &PasswordValidationConfig::default())
    }

    /// 验证密码强度（使用自定义配置）
    pub fn validate_with_config(
        password: &str,
        config: &PasswordValidationConfig,
    ) -> Result<(), PasswordError> {
        // 长度检查
        if password.len() < config.min_length {
            return Err(PasswordError::TooShort(config.min_length));
        }

        if password.len() > config.max_length {
            return Err(PasswordError::TooLong(config.max_length));
        }

        // 常见密码检查
        if config.check_common_passwords {
            let lowercase_password = password.to_lowercase();
            if COMMON_PASSWORDS
                .iter()
                .any(|&common| lowercase_password.contains(common))
            {
                return Err(PasswordError::CommonPassword);
            }
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

        if complexity_count < config.min_complexity_types {
            return Err(PasswordError::TooWeak {
                required_types: config.min_complexity_types,
                found_types: complexity_count,
            });
        }

        // 强度评分检查
        let score = Self::calculate_strength_score(password);
        let strength = PasswordStrength::from_score(score);

        if (strength as u8) < (config.min_strength as u8) {
            return Err(PasswordError::InsufficientStrength {
                required: config.min_strength,
                actual: strength,
            });
        }

        Ok(())
    }

    /// 计算密码强度分数（0-100）
    pub fn calculate_strength_score(password: &str) -> u8 {
        let mut score = 0u8;

        // 长度分数（最多 30 分）
        let length_score = (password.len() as u8).min(30);
        score += length_score;

        // 字符类型多样性（最多 40 分）
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if has_lowercase {
            score += 10;
        }
        if has_uppercase {
            score += 10;
        }
        if has_digit {
            score += 10;
        }
        if has_special {
            score += 10;
        }

        // 字符多样性（最多 20 分）
        let unique_chars = password
            .chars()
            .collect::<std::collections::HashSet<_>>()
            .len();
        let diversity_score = ((unique_chars as f32 / password.len() as f32) * 20.0) as u8;
        score += diversity_score;

        // 没有重复模式（最多 10 分）
        if !Self::has_repeated_patterns(password) {
            score += 10;
        }

        score.min(100)
    }

    /// 检查是否有重复模式（如 "aaa", "123", "abc"）
    fn has_repeated_patterns(password: &str) -> bool {
        let chars: Vec<char> = password.chars().collect();

        for window in chars.windows(3) {
            // 检查连续相同字符
            if window[0] == window[1] && window[1] == window[2] {
                return true;
            }

            // 检查连续递增/递减
            if match (
                window[0].to_digit(36),
                window[1].to_digit(36),
                window[2].to_digit(36),
            ) {
                (Some(a), Some(b), Some(c)) => {
                    (b == a + 1 && c == b + 1) || (b + 1 == a && c + 1 == b)
                }
                _ => false,
            } {
                return true;
            }
        }

        false
    }

    /// 获取密码强度等级
    pub fn get_strength(password: &str) -> PasswordStrength {
        let score = Self::calculate_strength_score(password);
        PasswordStrength::from_score(score)
    }
}

/// Password 错误
#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Password is too short (minimum {0} characters)")]
    TooShort(usize),

    #[error("Password is too long (maximum {0} characters)")]
    TooLong(usize),

    #[error(
        "Password is too weak (requires {required_types} character types, found {found_types})"
    )]
    TooWeak {
        required_types: usize,
        found_types: usize,
    },

    #[error("Password is a common/weak password")]
    CommonPassword,

    #[error("Password strength insufficient (required: {required:?}, actual: {actual:?})")]
    InsufficientStrength {
        required: PasswordStrength,
        actual: PasswordStrength,
    },

    #[error("Password hashing failed: {0}")]
    HashingFailed(String),

    #[error("Invalid password hash: {0}")]
    InvalidHash(String),
}

impl From<PasswordError> for errors::AppError {
    fn from(err: PasswordError) -> Self {
        errors::AppError::validation(err.to_string())
    }
}
