//! Access Token 仓储接口

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;

use crate::oauth::domain::entities::{AccessToken, OAuthClientId};

/// Access Token 仓储接口
#[async_trait]
pub trait AccessTokenRepository: Send + Sync {
    /// 根据 Token 查找
    async fn find_by_token(
        &self,
        token: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<AccessToken>>;

    /// 保存 Token
    async fn save(&self, token: &AccessToken) -> AppResult<()>;

    /// 更新 Token（撤销）
    async fn update(&self, token: &AccessToken) -> AppResult<()>;

    /// 删除 Token
    async fn delete(&self, token: &str, tenant_id: &TenantId) -> AppResult<()>;

    /// 删除过期的 Token
    async fn delete_expired(&self, tenant_id: &TenantId) -> AppResult<u64>;

    /// 删除用户的所有 Token
    async fn delete_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<u64>;

    /// 删除 Client 的所有 Token
    async fn delete_by_client_id(
        &self,
        client_id: &OAuthClientId,
        tenant_id: &TenantId,
    ) -> AppResult<u64>;

    /// 列出用户的活跃 Token
    async fn list_active_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<AccessToken>>;
}
