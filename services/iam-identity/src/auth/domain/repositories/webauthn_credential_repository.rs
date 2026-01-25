//! WebAuthn 凭证仓储接口

use async_trait::async_trait;
use cuba_errors::AppResult;
use uuid::Uuid;

use crate::auth::domain::entities::{WebAuthnCredential, WebAuthnCredentialId};

/// WebAuthn 凭证仓储接口
#[async_trait]
pub trait WebAuthnCredentialRepository: Send + Sync {
    /// 保存凭证
    async fn save(&self, credential: &WebAuthnCredential) -> AppResult<()>;

    /// 根据 ID 查找凭证
    async fn find_by_id(&self, id: &WebAuthnCredentialId) -> AppResult<Option<WebAuthnCredential>>;

    /// 根据凭证 ID 查找
    async fn find_by_credential_id(&self, credential_id: &[u8]) -> AppResult<Option<WebAuthnCredential>>;

    /// 根据用户 ID 查找所有凭证
    async fn find_by_user_id(&self, user_id: &Uuid) -> AppResult<Vec<WebAuthnCredential>>;

    /// 更新凭证
    async fn update(&self, credential: &WebAuthnCredential) -> AppResult<()>;

    /// 删除凭证
    async fn delete(&self, id: &WebAuthnCredentialId) -> AppResult<()>;

    /// 检查用户是否有 WebAuthn 凭证
    async fn has_credentials(&self, user_id: &Uuid) -> AppResult<bool>;
}
