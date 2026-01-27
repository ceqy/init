//! OAuth Client 仓储接口

use async_trait::async_trait;
use cuba_common::TenantId;
use cuba_errors::AppResult;

use crate::domain::oauth::{OAuthClient, OAuthClientId};

/// OAuth Client 仓储接口
#[async_trait]
pub trait OAuthClientRepository: Send + Sync {
    /// 根据 ID 查找 Client
    async fn find_by_id(
        &self,
        id: &OAuthClientId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<OAuthClient>>;

    /// 保存 Client
    async fn save(&self, client: &OAuthClient) -> AppResult<()>;

    /// 更新 Client
    async fn update(&self, client: &OAuthClient) -> AppResult<()>;

    /// 删除 Client
    async fn delete(&self, id: &OAuthClientId, tenant_id: &TenantId) -> AppResult<()>;

    /// 检查 Client 是否存在
    async fn exists(&self, id: &OAuthClientId, tenant_id: &TenantId) -> AppResult<bool>;

    /// 列出租户的所有 Client
    async fn list_by_tenant(
        &self,
        tenant_id: &TenantId,
        page: i64,
        page_size: i64,
    ) -> AppResult<Vec<OAuthClient>>;

    /// 统计租户的 Client 数量
    async fn count_by_tenant(&self, tenant_id: &TenantId) -> AppResult<i64>;
}
