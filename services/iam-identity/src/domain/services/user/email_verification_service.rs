//! 邮箱验证服务

use std::sync::Arc;

use cuba_adapter_email::EmailSender;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use tracing::{debug, info, warn};

use crate::domain::repositories::user::{EmailVerificationRepository, UserRepository};
use crate::domain::user::EmailVerification;

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
        let verification =
            EmailVerification::new(user_id.clone(), tenant_id.clone(), user.email.to_string());

        // 6. 保存验证记录
        self.email_verification_repo.save(&verification).await?;

        // 7. 发送邮件
        let subject = "邮箱验证码";
        let body = format!(
            "您好，{}！\n\n您的邮箱验证码是：{}\n\n验证码将在 {} 分钟后失效。\n\n如果您没有请求验证，请忽略此邮件。",
            user.username, verification.code, 10
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
    /// - `uow`: 工作单元
    /// - `user_id`: 用户 ID
    /// - `code`: 验证码
    /// - `tenant_id`: 租户 ID
    pub async fn verify_code(
        &self,
        uow: &dyn crate::domain::unit_of_work::UnitOfWork,
        user_id: &UserId,
        code: &str,
        tenant_id: &TenantId,
    ) -> AppResult<()> {
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Verifying email code");

        let email_repo = uow.email_verifications();
        let user_repo = uow.users();

        // 1. 查找最新的验证记录
        let mut verification = email_repo
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
        email_repo.update(&verification).await?;

        // 4. 更新用户的 email_verified 字段
        let mut user = user_repo
            .find_by_id(user_id, tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        user.email_verified = true;
        user_repo.update(&user).await?;

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

        let deleted = self
            .email_verification_repo
            .delete_expired(tenant_id)
            .await?;

        info!(tenant_id = %tenant_id, deleted = deleted, "Expired email verifications cleaned up");
        Ok(deleted)
    }
}
