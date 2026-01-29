//! 邮箱验证仓储接口

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;

use crate::domain::user::{EmailVerification, EmailVerificationId};

/// 邮箱验证仓储接口
#[async_trait]
pub trait EmailVerificationRepository: Send + Sync {
    /// 保存邮箱验证
    async fn save(&self, verification: &EmailVerification) -> AppResult<()>;

    /// 根据 ID 查找
    async fn find_by_id(
        &self,
        id: &EmailVerificationId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<EmailVerification>>;

    /// 根据用户 ID 查找最新的验证记录
    async fn find_latest_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<EmailVerification>>;

    /// 根据邮箱查找最新的验证记录
    async fn find_latest_by_email(
        &self,
        email: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<EmailVerification>>;

    /// 更新验证记录
    async fn update(&self, verification: &EmailVerification) -> AppResult<()>;

    /// 删除过期的验证记录
    async fn delete_expired(&self, tenant_id: &TenantId) -> AppResult<u64>;

    /// 删除所有过期的验证记录 (全局清理)
    async fn delete_all_expired(&self) -> AppResult<u64>;

    /// 统计用户今天发送的验证码数量（防止滥用）
    async fn count_today_by_user(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<i64>;
}
