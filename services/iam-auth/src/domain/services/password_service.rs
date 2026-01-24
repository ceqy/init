//! 密码服务

use cuba_errors::AppResult;

use crate::domain::value_objects::HashedPassword;

/// 密码服务
pub struct PasswordService;

impl PasswordService {
    /// 哈希密码
    pub fn hash_password(password: &str) -> AppResult<HashedPassword> {
        HashedPassword::from_plain(password)
    }

    /// 验证密码
    pub fn verify_password(password: &str, hash: &HashedPassword) -> AppResult<bool> {
        hash.verify(password)
    }
}
