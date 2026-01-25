//! 会话 Repository trait

use async_trait::async_trait;
use cuba_common::UserId;
use cuba_errors::AppResult;

use crate::auth::domain::entities::{Session, SessionId};

#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// 根据 ID 查找会话
    async fn find_by_id(&self, id: &SessionId) -> AppResult<Option<Session>>;

    /// 根据刷新令牌哈希查找会话
    async fn find_by_refresh_token_hash(&self, hash: &str) -> AppResult<Option<Session>>;

    /// 获取用户的所有活跃会话
    async fn find_active_by_user_id(&self, user_id: &UserId) -> AppResult<Vec<Session>>;

    /// 保存会话
    async fn save(&self, session: &Session) -> AppResult<()>;

    /// 更新会话
    async fn update(&self, session: &Session) -> AppResult<()>;

    /// 删除会话
    async fn delete(&self, id: &SessionId) -> AppResult<()>;

    /// 撤销用户的所有会话
    async fn revoke_all_by_user_id(&self, user_id: &UserId) -> AppResult<()>;

    /// 清理过期会话
    async fn cleanup_expired(&self) -> AppResult<u64>;
}
