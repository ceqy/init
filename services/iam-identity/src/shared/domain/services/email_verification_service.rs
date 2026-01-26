//! 邮箱验证服务

use std::sync::Arc;

use cuba_adapter_email::EmailSender;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use tracing::{debug, info, warn};

use crate::shared::domain::entities::EmailVerification;
use crate::shared::domain::repositories::{EmailVerificationRepository, UserRepository};

/// 邮箱验证服务
pub struct EmailVerificationService {
    email_verification_repo: Arc<dyn EmailVerificationRepository>,
    user_repo: Arc<dyn UserRepository>,
    email_sender: Arc<dyn EmailSender>,
}

impl EmailVerificationService {
    pub fn new(
        email_verification_repo: Arc<dyn EmailVerificationRepository>,
        user_repo: Arc<dyn UserRepository>,
        email_sender: Arc<dyn EmailSender>,
    ) -> Self {
        Self {
            email_verification_repo,
            user_repo,
            email_sender,
        }
    }

    /// 发送邮箱验证码
    ///
    /// # 参数
    /// - `user_id`: 用户 ID
    /// - `tenant_id`: 租户 ID
    ///
    /// # 返回
    /// - 验证码有效期（秒）
    pub async fn send_verification_code(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<i64> {
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Sending email verification code");

        // 1. 检查今天发送次数（最多5次）
        let today_count = self
            .email_verification_repo
            .count_today_by_user(user_id, tenant_id)
            .await?;

        if today_count >= 5 {
            warn!(user_id = %user_id, today_count = today_count, "Too many verification codes sent today");
            return Err(AppError::resource_exhausted(
                "Too many verification codes sent today. Please try again tomorrow.",
            ));
        }

        // 2. 查找用户
        let user = self
            .user_repo
            .find_by_id(user_id, tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        // 3. 检查用户状态
        if user.status.to_string() != "Active" {
            warn!(user_id = %user_id, status = %user.status, "Inactive user attempted email verification");
            return Err(AppError::failed_precondition("User account is not active"));
        }

        // 4. 检查邮箱是否已验证
        if user.email_verified {
            return Err(AppError::failed_precondition("Email already verified"));
        }

        // 5. 创建验证记录
        let verification = EmailVerification::new(
            user_id.clone(),
            tenant_id.clone(),
            user.email.to_string(),
        );

        // 6. 保存验证记录
        self.email_verification_repo.save(&verification).await?;

        // 7. 发送邮件
        let subject = "邮箱验证码";
        let body = format!(
            "您好，{}！\n\n您的邮箱验证码是：{}\n\n验证码将在 {} 分钟后失效。\n\n如果您没有请求验证，请忽略此邮件。",
            user.username,
            verification.code,
            10
        );

        self.email_sender
            .send_text_email(&user.email.to_string(), subject, &body)
            .await?;

        info!(
            user_id = %user_id,
            verification_id = %verification.id,
            expires_at = %verification.expires_at,
            "Email verification code sent"
        );

        Ok(verification.get_remaining_seconds())
    }

    /// 验证邮箱验证码
    ///
    /// # 参数
    /// - `user_id`: 用户 ID
    /// - `code`: 验证码
    /// - `tenant_id`: 租户 ID
    pub async fn verify_code(
        &self,
        user_id: &UserId,
        code: &str,
        tenant_id: &TenantId,
    ) -> AppResult<()> {
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Verifying email code");

        // 1. 查找最新的验证记录
        let mut verification = self
            .email_verification_repo
            .find_latest_by_user_id(user_id, tenant_id)
            .await?
            .ok_or_else(|| {
                warn!(user_id = %user_id, "No verification record found");
                AppError::not_found("No verification code found. Please request a new one.")
            })?;

        // 2. 验证验证码
        verification.verify(code).map_err(|e| {
            warn!(user_id = %user_id, error = %e, "Email verification failed");
            AppError::unauthenticated(e.to_string())
        })?;

        // 3. 更新验证记录
        self.email_verification_repo.update(&verification).await?;

        // 4. 更新用户的 email_verified 字段
        let mut user = self
            .user_repo
            .find_by_id(user_id, tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        user.email_verified = true;
        self.user_repo.update(&user).await?;

        info!(
            user_id = %user_id,
            verification_id = %verification.id,
            "Email verified successfully"
        );

        Ok(())
    }

    /// 清理过期的验证记录
    ///
    /// # 参数
    /// - `tenant_id`: 租户 ID
    ///
    /// # 返回
    /// - 删除的记录数量
    pub async fn cleanup_expired(&self, tenant_id: &TenantId) -> AppResult<u64> {
        debug!(tenant_id = %tenant_id, "Cleaning up expired email verifications");

        let deleted = self.email_verification_repo.delete_expired(tenant_id).await?;

        info!(tenant_id = %tenant_id, deleted = deleted, "Expired email verifications cleaned up");
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::domain::entities::User;
    use crate::shared::domain::value_objects::{Email, Password, Username};
    use crate::shared::infrastructure::persistence::{
        PostgresEmailVerificationRepository, PostgresUserRepository,
    };
    use uuid::Uuid;

    // Mock EmailSender for testing
    struct MockEmailSender;

    #[async_trait::async_trait]
    impl EmailSender for MockEmailSender {
        async fn send_text_email(&self, _to: &str, _subject: &str, _body: &str) -> AppResult<()> {
            Ok(())
        }

        async fn send_html_email(
            &self,
            _to: &str,
            _subject: &str,
            _html_body: &str,
            _text_body: Option<&str>,
        ) -> AppResult<()> {
            Ok(())
        }

        async fn send_template_email(
            &self,
            _to: &str,
            _subject: &str,
            _template_name: &str,
            _context: &serde_json::Value,
        ) -> AppResult<()> {
            Ok(())
        }
    }

    #[sqlx::test]
    async fn test_send_and_verify_email_code(pool: sqlx::PgPool) {
        let email_verification_repo =
            Arc::new(PostgresEmailVerificationRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let email_sender = Arc::new(MockEmailSender);

        let service = EmailVerificationService::new(
            email_verification_repo.clone(),
            user_repo.clone(),
            email_sender,
        );

        let tenant_id = TenantId::from_uuid(Uuid::new_v4());
        let email = Email::new("test@example.com").unwrap();
        let username = Username::new("testuser").unwrap();
        let password = Password::new("Password123!").unwrap();

        // 创建测试用户
        let user = User::new(username, email, password, tenant_id.clone()).unwrap();
        user_repo.save(&user, &tenant_id).await.unwrap();

        // 发送验证码
        let expires_in = service
            .send_verification_code(&user.id, &tenant_id)
            .await
            .unwrap();
        assert!(expires_in > 0);

        // 获取验证码
        let verification = email_verification_repo
            .find_latest_by_user_id(&user.id, &tenant_id)
            .await
            .unwrap()
            .unwrap();

        // 验证验证码
        service
            .verify_code(&user.id, &verification.code, &tenant_id)
            .await
            .unwrap();

        // 检查用户的 email_verified 字段
        let updated_user = user_repo
            .find_by_id(&user.id, &tenant_id)
            .await
            .unwrap()
            .unwrap();
        assert!(updated_user.email_verified);
    }

    #[sqlx::test]
    async fn test_prevent_too_many_sends(pool: sqlx::PgPool) {
        let email_verification_repo =
            Arc::new(PostgresEmailVerificationRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let email_sender = Arc::new(MockEmailSender);

        let service = EmailVerificationService::new(
            email_verification_repo,
            user_repo.clone(),
            email_sender,
        );

        let tenant_id = TenantId::from_uuid(Uuid::new_v4());
        let email = Email::new("test@example.com").unwrap();
        let username = Username::new("testuser").unwrap();
        let password = Password::new("Password123!").unwrap();

        // 创建测试用户
        let user = User::new(username, email, password, tenant_id.clone()).unwrap();
        user_repo.save(&user, &tenant_id).await.unwrap();

        // 发送 5 次验证码（最大限制）
        for _ in 0..5 {
            service
                .send_verification_code(&user.id, &tenant_id)
                .await
                .unwrap();
        }

        // 第 6 次应该失败
        let result = service.send_verification_code(&user.id, &tenant_id).await;
        assert!(result.is_err());
    }
}
