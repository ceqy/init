//! 密码重置服务

use std::sync::Arc;


use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use rand::Rng;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::domain::auth::PasswordResetToken;
use crate::domain::repositories::auth::PasswordResetRepository;
use crate::domain::repositories::user::UserRepository;
use crate::domain::value_objects::Email;

/// 密码重置服务
pub struct PasswordResetService {
    password_reset_repo: Arc<dyn PasswordResetRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl PasswordResetService {
    pub fn new(
        password_reset_repo: Arc<dyn PasswordResetRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            password_reset_repo,
            user_repo,
        }
    }

    /// 生成密码重置令牌
    ///
    /// # 参数
    /// - `email`: 用户邮箱
    /// - `tenant_id`: 租户 ID
    /// - `expires_in_minutes`: 过期时间（分钟）
    ///
    /// # 返回
    /// - `(token_string, user_id)`: 令牌字符串和用户 ID
    pub async fn generate_reset_token(
        &self,
        email: &Email,
        tenant_id: &TenantId,
        expires_in_minutes: i64,
    ) -> AppResult<(String, UserId)> {
        debug!(email = %email, tenant_id = %tenant_id, "Generating password reset token");

        // 1. 查找用户
        let user = self
            .user_repo
            .find_by_email(email, tenant_id)
            .await?
            .ok_or_else(|| {
                // 为了安全，不暴露用户是否存在
                debug!(email = %email, "User not found for password reset");
                AppError::not_found("User not found")
            })?;

        // 2. 检查用户状态
        if user.status.to_string() != "Active" {
            warn!(user_id = %user.id, status = %user.status, "Inactive user attempted password reset");
            return Err(AppError::failed_precondition("User account is not active"));
        }

        // 3. 检查是否有太多未使用的令牌（防止滥用）
        let unused_count = self
            .password_reset_repo
            .count_unused_by_user_id(&user.id, tenant_id)
            .await?;

        if unused_count >= 3 {
            warn!(user_id = %user.id, unused_count = unused_count, "Too many unused password reset tokens");
            return Err(AppError::resource_exhausted(
                "Too many pending password reset requests. Please wait before requesting again.",
            ));
        }

        // 4. 生成随机令牌（32 字节 = 64 个十六进制字符）
        let token_bytes: [u8; 32] = rand::thread_rng().r#gen();
        let token_string = hex::encode(token_bytes);

        // 5. 计算令牌哈希（存储哈希而不是原始令牌）
        let mut hasher = Sha256::new();
        hasher.update(token_string.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        // 6. 创建令牌实体
        let reset_token = PasswordResetToken::new(user.id.clone(), user.tenant_id.clone(), token_hash, expires_in_minutes);

        // 7. 保存令牌
        self.password_reset_repo.save(&reset_token).await?;

        info!(
            user_id = %user.id,
            token_id = %reset_token.id,
            expires_at = %reset_token.expires_at,
            "Password reset token generated"
        );

        Ok((token_string, user.id))
    }

    /// 验证密码重置令牌
    ///
    /// # 参数
    /// - `token_string`: 令牌字符串
    /// - `tenant_id`: 租户 ID
    ///
    /// # 返回
    /// - `user_id`: 用户 ID
    pub async fn verify_reset_token(
        &self,
        token_string: &str,
        tenant_id: &TenantId,
    ) -> AppResult<UserId> {
        debug!(tenant_id = %tenant_id, "Verifying password reset token");

        // 1. 计算令牌哈希
        let mut hasher = Sha256::new();
        hasher.update(token_string.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        // 2. 查找令牌
        let mut token = self
            .password_reset_repo
            .find_by_token_hash(&token_hash, tenant_id)
            .await?
            .ok_or_else(|| {
                warn!("Invalid password reset token");
                AppError::unauthenticated("Invalid or expired reset token")
            })?;

        // 3. 验证令牌状态
        if !token.is_valid() {
            warn!(token_id = %token.id, used = token.used, expired = token.is_expired(), "Invalid token state");
            return Err(AppError::unauthenticated("Invalid or expired reset token"));
        }

        // 4. 标记令牌为已使用
        token.mark_as_used();
        self.password_reset_repo.update(&token).await?;

        info!(
            user_id = %token.user_id,
            token_id = %token.id,
            "Password reset token verified and marked as used"
        );

        Ok(token.user_id)
    }

    /// 撤销用户的所有密码重置令牌
    ///
    /// # 参数
    /// - `user_id`: 用户 ID
    /// - `tenant_id`: 租户 ID
    pub async fn revoke_all_tokens(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<()> {
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Revoking all password reset tokens");

        self.password_reset_repo
            .delete_by_user_id(user_id, tenant_id)
            .await?;

        info!(user_id = %user_id, "All password reset tokens revoked");
        Ok(())
    }

    /// 清理过期的令牌
    ///
    /// # 参数
    /// - `tenant_id`: 租户 ID
    ///
    /// # 返回
    /// - 删除的令牌数量
    pub async fn cleanup_expired_tokens(&self, tenant_id: &TenantId) -> AppResult<u64> {
        debug!(tenant_id = %tenant_id, "Cleaning up expired password reset tokens");

        let deleted = self.password_reset_repo.delete_expired(tenant_id).await?;

        info!(tenant_id = %tenant_id, deleted = deleted, "Expired tokens cleaned up");
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::persistence::auth::PostgresPasswordResetRepository;
    use crate::domain::user::User;
    use crate::domain::value_objects::{Password, Username};
    use crate::infrastructure::persistence::user::PostgresUserRepository;
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_generate_and_verify_reset_token(pool: sqlx::PgPool) {
        let password_reset_repo = Arc::new(PostgresPasswordResetRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let service = PasswordResetService::new(password_reset_repo, user_repo.clone());

        let tenant_id = TenantId::from_uuid(Uuid::new_v4());
        let email = Email::new("test@example.com").unwrap();
        let username = Username::new("testuser").unwrap();
        let password = Password::new("Password123!").unwrap();

        // 创建测试用户
        let user = User::new(
            username,
            email.clone(),
            password,
            tenant_id.clone(),
        )
        .unwrap();
        user_repo.save(&user, &tenant_id).await.unwrap();

        // 生成令牌
        let (token_string, user_id) = service
            .generate_reset_token(&email, &tenant_id, 15)
            .await
            .unwrap();

        assert_eq!(user_id, user.id);
        assert_eq!(token_string.len(), 64); // 32 字节 = 64 个十六进制字符

        // 验证令牌
        let verified_user_id = service
            .verify_reset_token(&token_string, &tenant_id)
            .await
            .unwrap();

        assert_eq!(verified_user_id, user.id);

        // 再次验证应该失败（已使用）
        let result = service.verify_reset_token(&token_string, &tenant_id).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_prevent_too_many_tokens(pool: sqlx::PgPool) {
        let password_reset_repo = Arc::new(PostgresPasswordResetRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let service = PasswordResetService::new(password_reset_repo, user_repo.clone());

        let tenant_id = TenantId::from_uuid(Uuid::new_v4());
        let email = Email::new("test@example.com").unwrap();
        let username = Username::new("testuser").unwrap();
        let password = Password::new("Password123!").unwrap();

        // 创建测试用户
        let user = User::new(username, email.clone(), password, tenant_id.clone()).unwrap();
        user_repo.save(&user, &tenant_id).await.unwrap();

        // 生成 3 个令牌（最大限制）
        for _ in 0..3 {
            service
                .generate_reset_token(&email, &tenant_id, 15)
                .await
                .unwrap();
        }

        // 第 4 个应该失败
        let result = service.generate_reset_token(&email, &tenant_id, 15).await;
        assert!(result.is_err());
    }
}
