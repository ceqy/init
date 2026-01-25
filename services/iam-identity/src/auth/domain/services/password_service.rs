//! 密码服务

use cuba_errors::AppResult;

use crate::shared::domain::value_objects::HashedPassword;

/// 密码服务
pub struct PasswordService;

impl PasswordService {
    /// 哈希密码
    pub fn hash_password(password: &str) -> AppResult<HashedPassword> {
        HashedPassword::from_plain(password).map_err(Into::into)
    }

    /// 验证密码
    pub fn verify_password(password: &str, hash: &HashedPassword) -> AppResult<bool> {
        hash.verify(password).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_success() {
        let result = PasswordService::hash_password("Test1234!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_hash_password_weak() {
        let result = PasswordService::hash_password("weak");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_correct() {
        let hashed = PasswordService::hash_password("Test1234!").unwrap();
        let result = PasswordService::verify_password("Test1234!", &hashed);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let hashed = PasswordService::hash_password("Test1234!").unwrap();
        let result = PasswordService::verify_password("WrongPassword!", &hashed);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_hash_different_passwords_produce_different_hashes() {
        let hash1 = PasswordService::hash_password("Test1234!").unwrap();
        let hash2 = PasswordService::hash_password("Test1234!").unwrap();
        // 即使密码相同，由于盐不同，哈希也应该不同
        assert_ne!(hash1.0, hash2.0);
    }
}
