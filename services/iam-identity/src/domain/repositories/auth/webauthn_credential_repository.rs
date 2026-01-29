//! WebAuthn 凭证仓储接口

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;

use crate::domain::auth::{WebAuthnCredential, WebAuthnCredentialId};

/// WebAuthn 凭证仓储接口
#[async_trait]
pub trait WebAuthnCredentialRepository: Send + Sync {
    /// 保存凭证（自动使用凭证的 tenant_id）
    async fn save(&self, credential: &WebAuthnCredential) -> AppResult<()>;

    /// 根据 ID 查找凭证（带租户隔离）
    async fn find_by_id(
        &self,
        id: &WebAuthnCredentialId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<WebAuthnCredential>>;

    /// 根据凭证 ID 查找（带租户隔离）
    async fn find_by_credential_id(
        &self,
        credential_id: &[u8],
        tenant_id: &TenantId,
    ) -> AppResult<Option<WebAuthnCredential>>;

    /// 根据用户 ID 查找所有凭证（带租户隔离）
    async fn find_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<WebAuthnCredential>>;

    /// 更新凭证（验证 tenant_id 匹配）
    async fn update(&self, credential: &WebAuthnCredential) -> AppResult<()>;

    /// 删除凭证（带租户隔离）
    async fn delete(&self, id: &WebAuthnCredentialId, tenant_id: &TenantId) -> AppResult<()>;

    /// 检查用户是否有 WebAuthn 凭证（带租户隔离）
    async fn has_credentials(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<bool>;
}
