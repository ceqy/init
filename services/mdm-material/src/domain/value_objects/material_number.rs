//! 物料编号值对象

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 物料编号最大长度
const MAX_LENGTH: usize = 40;

/// 物料编号错误
#[derive(Debug, Error)]
pub enum MaterialNumberError {
    #[error("物料编号不能为空")]
    Empty,
    #[error("物料编号长度不能超过 {MAX_LENGTH} 个字符")]
    TooLong,
    #[error("物料编号包含无效字符: {0}")]
    InvalidCharacter(char),
}

/// 物料编号值对象
///
/// 业务规则:
/// - 不能为空
/// - 最大长度 40 字符
/// - 只允许字母、数字、连字符和下划线
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialNumber(String);

impl MaterialNumber {
    /// 创建新的物料编号
    pub fn new(number: impl Into<String>) -> Result<Self, MaterialNumberError> {
        let number = number.into().trim().to_uppercase();

        if number.is_empty() {
            return Err(MaterialNumberError::Empty);
        }

        if number.len() > MAX_LENGTH {
            return Err(MaterialNumberError::TooLong);
        }

        // 验证字符
        for c in number.chars() {
            if !c.is_alphanumeric() && c != '-' && c != '_' {
                return Err(MaterialNumberError::InvalidCharacter(c));
            }
        }

        Ok(Self(number))
    }

    /// 获取物料编号字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 转换为字符串
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for MaterialNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for MaterialNumber {
    type Error = MaterialNumberError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for MaterialNumber {
    type Error = MaterialNumberError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_material_number() {
        let number = MaterialNumber::new("MAT-001").unwrap();
        assert_eq!(number.as_str(), "MAT-001");
    }

    #[test]
    fn test_uppercase_conversion() {
        let number = MaterialNumber::new("mat-001").unwrap();
        assert_eq!(number.as_str(), "MAT-001");
    }

    #[test]
    fn test_empty_number() {
        let result = MaterialNumber::new("");
        assert!(matches!(result, Err(MaterialNumberError::Empty)));
    }

    #[test]
    fn test_too_long_number() {
        let long_number = "A".repeat(41);
        let result = MaterialNumber::new(long_number);
        assert!(matches!(result, Err(MaterialNumberError::TooLong)));
    }

    #[test]
    fn test_invalid_character() {
        let result = MaterialNumber::new("MAT@001");
        assert!(matches!(result, Err(MaterialNumberError::InvalidCharacter('@'))));
    }
}
