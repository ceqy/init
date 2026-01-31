//! 密码重置令牌仓储接口

use async_trait::async_trait;
use common::{TenantId, UserId};
use errors::AppResult;

use crate::domain::auth::{PasswordResetToken, PasswordResetTokenId};

/// 密码重置令牌仓储接口
#[async_trait]
pub trait PasswordResetRepository: Send + Sync {
    /// 保存密码重置令牌（自动使用令牌的 tenant_id）
    async fn save(&self, token: &PasswordResetToken) -> AppResult<()>;

    /// 根据 ID 查找令牌（带租户隔离）
    async fn find_by_id(
        &self,
        id: &PasswordResetTokenId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PasswordResetToken>>;

    /// 根据令牌哈希查找（带租户隔离）
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PasswordResetToken>>;

    /// 更新令牌（验证 tenant_id 匹配）
    async fn update(&self, token: &PasswordResetToken) -> AppResult<()>;

    /// 标记令牌为已使用（带租户隔离）
    async fn mark_as_used(&self, id: &PasswordResetTokenId, tenant_id: &TenantId) -> AppResult<()>;

    /// 删除用户的所有令牌（带租户隔离）
    async fn delete_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<()>;

    /// 删除过期的令牌（带租户隔离）
    async fn delete_expired(&self, tenant_id: &TenantId) -> AppResult<u64>;

    /// 删除所有过期的令牌 (全局清理)
    async fn delete_all_expired(&self) -> AppResult<u64>;

    /// 统计用户未使用的令牌数量（带租户隔离）
    async fn count_unused_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<i64>;
}
