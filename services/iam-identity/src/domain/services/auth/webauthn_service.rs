//! WebAuthn 服务

use cuba_common::TenantId;
use cuba_errors::{AppError, AppResult};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::domain::auth::{WebAuthnCredential, WebAuthnCredentialId};
use crate::domain::repositories::auth::WebAuthnCredentialRepository;

/// WebAuthn 服务
pub struct WebAuthnService {
    webauthn: Arc<Webauthn>,
    credential_repo: Arc<dyn WebAuthnCredentialRepository>,
}

impl WebAuthnService {
    /// 创建新的 WebAuthn 服务
    pub fn new(
        rp_id: String,
        rp_origin: Url,
        credential_repo: Arc<dyn WebAuthnCredentialRepository>,
    ) -> AppResult<Self> {
        let builder = WebauthnBuilder::new(&rp_id, &rp_origin)
            .map_err(|e| AppError::internal(format!("Failed to create WebAuthn builder: {}", e)))?;

        let webauthn = Arc::new(
            builder
                .build()
                .map_err(|e| AppError::internal(format!("Failed to build WebAuthn: {}", e)))?,
        );

        Ok(Self {
            webauthn,
            credential_repo,
        })
    }

    /// 开始注册流程
    pub async fn start_registration(
        &self,
        user_id: Uuid,
        username: &str,
        display_name: &str,
        tenant_id: &TenantId,
    ) -> AppResult<(CreationChallengeResponse, PasskeyRegistration)> {
        info!("Starting WebAuthn registration for user: {}", user_id);

        // 获取用户现有的凭证
        let user_id_typed = cuba_common::UserId::from_uuid(user_id);
        let existing_credentials = self.credential_repo.find_by_user_id(&user_id_typed, tenant_id).await?;

        // 转换为 CredentialID 格式
        let exclude_credentials: Vec<CredentialID> = existing_credentials
            .iter()
            .filter_map(|c| CredentialID::try_from(c.credential_id.clone()).ok())
            .collect();

        // 创建注册挑战
        let (ccr, reg_state) = self
            .webauthn
            .start_passkey_registration(
                Uuid::from_bytes(*user_id.as_bytes()),
                username,
                display_name,
                Some(exclude_credentials),
            )
            .map_err(|e| AppError::internal(format!("Failed to start registration: {}", e)))?;

        debug!("Registration challenge created");
        Ok((ccr, reg_state))
    }

    /// 完成注册流程
    pub async fn finish_registration(
        &self,
        user_id: Uuid,
        credential_name: String,
        reg: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
        tenant_id: &TenantId,
    ) -> AppResult<WebAuthnCredential> {
        info!("Finishing WebAuthn registration for user: {}", user_id);

        // 验证注册响应
        let passkey = self
            .webauthn
            .finish_passkey_registration(reg, state)
            .map_err(|e| AppError::validation(format!("Registration verification failed: {}", e)))?;

        // 提取传输方式
        let transports: Vec<String> = reg
            .response
            .transports
            .as_ref()
            .map(|t| t.iter().map(|t| format!("{:?}", t)).collect())
            .unwrap_or_default();

        // 创建凭证实体
        let credential = WebAuthnCredential::from_passkey(
            user_id,
            credential_name,
            &passkey,
            None, // aaguid - not easily accessible in webauthn-rs 0.5
            transports,
            tenant_id.clone(),
        )
        .map_err(|e| AppError::internal(format!("Failed to create credential: {}", e)))?;

        // 保存凭证
        self.credential_repo.save(&credential).await?;

        info!("WebAuthn credential registered successfully");
        Ok(credential)
    }

    /// 开始认证流程
    pub async fn start_authentication(
        &self,
        user_id: Uuid,
        tenant_id: &TenantId,
    ) -> AppResult<(RequestChallengeResponse, PasskeyAuthentication)> {
        info!("Starting WebAuthn authentication for user: {}", user_id);

        // 获取用户的凭证
        let user_id_typed = cuba_common::UserId::from_uuid(user_id);
        let credentials = self.credential_repo.find_by_user_id(&user_id_typed, tenant_id).await?;

        if credentials.is_empty() {
            return Err(AppError::not_found("No WebAuthn credentials found"));
        }

        // 转换为 Passkey 格式
        let passkeys: Vec<Passkey> = credentials
            .iter()
            .filter_map(|c| c.to_passkey().ok())
            .collect();

        if passkeys.is_empty() {
            return Err(AppError::internal("Failed to load credentials"));
        }

        // 创建认证挑战
        let (rcr, auth_state) = self
            .webauthn
            .start_passkey_authentication(&passkeys)
            .map_err(|e| AppError::internal(format!("Failed to start authentication: {}", e)))?;

        debug!("Authentication challenge created");
        Ok((rcr, auth_state))
    }

    /// 完成认证流程
    pub async fn finish_authentication(
        &self,
        auth: &PublicKeyCredential,
        state: &PasskeyAuthentication,
        tenant_id: &TenantId,
    ) -> AppResult<Uuid> {
        info!("Finishing WebAuthn authentication");

        // 验证认证响应
        let auth_result = self
            .webauthn
            .finish_passkey_authentication(auth, state)
            .map_err(|e| AppError::validation(format!("Authentication verification failed: {}", e)))?;

        // 查找凭证
        let credential_id = auth_result.cred_id().as_ref();
        let mut credential = self
            .credential_repo
            .find_by_credential_id(credential_id, tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("Credential not found"))?;

        // 更新凭证
        credential.update_from_authentication(&auth_result);
        self.credential_repo.update(&credential).await?;

        info!("WebAuthn authentication successful for user: {}", credential.user_id);
        Ok(credential.user_id)
    }

    /// 获取用户的所有凭证
    pub async fn list_credentials(&self, user_id: Uuid, tenant_id: &TenantId) -> AppResult<Vec<WebAuthnCredential>> {
        let user_id_typed = cuba_common::UserId::from_uuid(user_id);
        self.credential_repo.find_by_user_id(&user_id_typed, tenant_id).await
    }

    /// 删除凭证
    pub async fn delete_credential(&self, user_id: Uuid, credential_id: Uuid, tenant_id: &TenantId) -> AppResult<()> {
        // 验证凭证属于该用户
        let cred_id = WebAuthnCredentialId::from_uuid(credential_id);
        let credential = self
            .credential_repo
            .find_by_id(&cred_id, tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("Credential not found"))?;

        if credential.user_id != user_id {
            return Err(AppError::forbidden("Cannot delete credential of another user"));
        }

        self.credential_repo.delete(&cred_id, tenant_id).await
    }

    /// 检查用户是否有 WebAuthn 凭证
    pub async fn has_credentials(&self, user_id: Uuid, tenant_id: &TenantId) -> AppResult<bool> {
        let user_id_typed = cuba_common::UserId::from_uuid(user_id);
        self.credential_repo.has_credentials(&user_id_typed, tenant_id).await
    }
}

