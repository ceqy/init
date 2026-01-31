//! 手机验证仓储接口

use async_trait::async_trait;
use common::{TenantId, UserId};
use errors::AppResult;

use crate::domain::user::{PhoneVerification, PhoneVerificationId};

/// 手机验证仓储接口
#[async_trait]
pub trait PhoneVerificationRepository: Send + Sync {
    /// 保存手机验证
    async fn save(&self, verification: &PhoneVerification) -> AppResult<()>;

    /// 根据 ID 查找
    async fn find_by_id(
        &self,
        id: &PhoneVerificationId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PhoneVerification>>;

    /// 根据用户 ID 查找最新的验证记录
    async fn find_latest_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PhoneVerification>>;

    /// 根据手机号查找最新的验证记录
    async fn find_latest_by_phone(
        &self,
        phone: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PhoneVerification>>;

    /// 更新验证记录
    async fn update(&self, verification: &PhoneVerification) -> AppResult<()>;

    /// 删除过期的验证记录
    async fn delete_expired(&self, tenant_id: &TenantId) -> AppResult<u64>;

    /// 删除所有过期的验证记录 (全局清理)
    async fn delete_all_expired(&self) -> AppResult<u64>;

    /// 统计用户今天发送的验证码数量（防止滥用）
    async fn count_today_by_user(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<i64>;
}
