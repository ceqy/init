//! 授权码仓储接口

use async_trait::async_trait;
use cuba_common::TenantId;
use cuba_errors::AppResult;

use crate::domain::oauth::{AuthorizationCode, OAuthClientId};

/// 授权码仓储接口
#[async_trait]
pub trait AuthorizationCodeRepository: Send + Sync {
    /// 根据授权码查找
    async fn find_by_code(
        &self,
        code: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<AuthorizationCode>>;

    /// 保存授权码
    async fn save(&self, authorization_code: &AuthorizationCode) -> AppResult<()>;

    /// 更新授权码（标记为已使用）
    async fn update(&self, authorization_code: &AuthorizationCode) -> AppResult<()>;

    /// 删除授权码
    async fn delete(&self, code: &str, tenant_id: &TenantId) -> AppResult<()>;

    /// 删除过期的授权码
    async fn delete_expired(&self, tenant_id: &TenantId) -> AppResult<u64>;

    /// 删除用户的所有授权码
    async fn delete_by_user_id(
        &self,
        user_id: &cuba_common::UserId,
        tenant_id: &TenantId,
    ) -> AppResult<u64>;

    /// 删除 Client 的所有授权码
    async fn delete_by_client_id(
        &self,
        client_id: &OAuthClientId,
        tenant_id: &TenantId,
    ) -> AppResult<u64>;
}
