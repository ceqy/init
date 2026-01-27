//! 密码服务

use cuba_errors::AppResult;

use crate::domain::value_objects::HashedPassword;

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

    /// 修改用户密码
    pub fn change_password(&self, user: &mut crate::domain::user::User, new_password: &str) -> AppResult<()> {
        let hashed = Self::hash_password(new_password)?;
        user.update_password(hashed);
        Ok(())
    }
}

