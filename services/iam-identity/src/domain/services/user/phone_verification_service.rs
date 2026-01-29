//! 手机验证服务

use std::sync::Arc;

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use tracing::{debug, info, warn};

use crate::domain::repositories::user::{PhoneVerificationRepository, UserRepository};
use crate::domain::user::PhoneVerification;

/// 短信发送器接口
#[async_trait]
pub trait SmsSender: Send + Sync {
    /// 发送验证码短信
    async fn send_verification_code(&self, phone: &str, code: &str) -> AppResult<()>;
}

/// 手机验证服务
pub struct PhoneVerificationService {
    phone_verification_repo: Arc<dyn PhoneVerificationRepository>,
    user_repo: Arc<dyn UserRepository>,
    sms_sender: Arc<dyn SmsSender>,
}

impl PhoneVerificationService {
    pub fn new(
        phone_verification_repo: Arc<dyn PhoneVerificationRepository>,
        user_repo: Arc<dyn UserRepository>,
        sms_sender: Arc<dyn SmsSender>,
    ) -> Self {
        Self {
            phone_verification_repo,
            user_repo,
            sms_sender,
        }
    }

    /// 发送手机验证码
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
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Sending phone verification code");

        // 1. 检查今天发送次数（最多5次）
        let today_count = self
            .phone_verification_repo
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
            warn!(user_id = %user_id, status = %user.status, "Inactive user attempted phone verification");
            return Err(AppError::failed_precondition("User account is not active"));
        }

        // 4. 检查手机号是否存在
        let phone = user
            .phone
            .as_ref()
            .ok_or_else(|| AppError::failed_precondition("User has no phone number"))?;

        // 5. 检查手机号是否已验证
        if user.phone_verified {
            return Err(AppError::failed_precondition("Phone already verified"));
        }

        // 6. 创建验证记录
        let verification =
            PhoneVerification::new(user_id.clone(), tenant_id.clone(), phone.clone());

        // 7. 保存验证记录
        self.phone_verification_repo.save(&verification).await?;

        // 8. 发送短信
        self.sms_sender
            .send_verification_code(phone, &verification.code)
            .await?;

        info!(
            user_id = %user_id,
            verification_id = %verification.id,
            expires_at = %verification.expires_at,
            "Phone verification code sent"
        );

        Ok(verification.get_remaining_seconds())
    }

    /// 验证手机验证码
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
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Verifying phone code");

        let phone_repo = uow.phone_verifications();
        let user_repo = uow.users();

        // 1. 查找最新的验证记录
        let mut verification = phone_repo
            .find_latest_by_user_id(user_id, tenant_id)
            .await?
            .ok_or_else(|| {
                warn!(user_id = %user_id, "No verification record found");
                AppError::not_found("No verification code found. Please request a new one.")
            })?;

        // 2. 验证验证码
        verification.verify(code).map_err(|e| {
            warn!(user_id = %user_id, error = %e, "Phone verification failed");
            AppError::unauthenticated(e.to_string())
        })?;

        // 3. 更新验证记录
        phone_repo.update(&verification).await?;

        // 4. 更新用户的 phone_verified 字段
        let mut user = user_repo
            .find_by_id(user_id, tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        user.phone_verified = true;
        user_repo.update(&user).await?;

        info!(
            user_id = %user_id,
            verification_id = %verification.id,
            "Phone verified successfully"
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
        debug!(tenant_id = %tenant_id, "Cleaning up expired phone verifications");

        let deleted = self
            .phone_verification_repo
            .delete_expired(tenant_id)
            .await?;

        info!(tenant_id = %tenant_id, deleted = deleted, "Expired phone verifications cleaned up");
        Ok(deleted)
    }
}
