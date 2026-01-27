//! 会话 Repository trait

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;

use crate::domain::auth::{Session, SessionId};

#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// 根据 ID 查找会话（带租户隔离）
    async fn find_by_id(&self, id: &SessionId, tenant_id: &TenantId) -> AppResult<Option<Session>>;

    /// 根据刷新令牌哈希查找会话（带租户隔离）
    async fn find_by_refresh_token_hash(&self, hash: &str, tenant_id: &TenantId) -> AppResult<Option<Session>>;

    /// 获取用户的所有活跃会话（带租户隔离）
    async fn find_active_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<Vec<Session>>;

    /// 保存会话（自动使用会话的 tenant_id）
    async fn save(&self, session: &Session) -> AppResult<()>;

    /// 更新会话（验证 tenant_id 匹配）
    async fn update(&self, session: &Session) -> AppResult<()>;

    /// 删除会话（带租户隔离）
    async fn delete(&self, id: &SessionId, tenant_id: &TenantId) -> AppResult<()>;

    /// 撤销用户的所有会话（带租户隔离）
    async fn revoke_all_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<()>;

    /// 清理过期会话（带租户隔离）
    async fn cleanup_expired(&self, tenant_id: &TenantId) -> AppResult<u64>;
}
