//! 密码重置令牌仓储接口

use async_trait::async_trait;
use cuba_common::UserId;
use cuba_errors::AppResult;

use crate::auth::domain::entities::{PasswordResetToken, PasswordResetTokenId};

/// 密码重置令牌仓储接口
#[async_trait]
pub trait PasswordResetRepository: Send + Sync {
    /// 保存密码重置令牌
    async fn save(&self, token: &PasswordResetToken) -> AppResult<()>;

    /// 根据 ID 查找令牌
    async fn find_by_id(&self, id: &PasswordResetTokenId) -> AppResult<Option<PasswordResetToken>>;

    /// 根据令牌哈希查找
    async fn find_by_token_hash(&self, token_hash: &str) -> AppResult<Option<PasswordResetToken>>;

    /// 更新令牌
    async fn update(&self, token: &PasswordResetToken) -> AppResult<()>;

    /// 标记令牌为已使用
    async fn mark_as_used(&self, id: &PasswordResetTokenId) -> AppResult<()>;

    /// 删除用户的所有令牌
    async fn delete_by_user_id(&self, user_id: &UserId) -> AppResult<()>;

    /// 删除过期的令牌
    async fn delete_expired(&self) -> AppResult<u64>;

    /// 统计用户未使用的令牌数量
    async fn count_unused_by_user_id(&self, user_id: &UserId) -> AppResult<i64>;
}
