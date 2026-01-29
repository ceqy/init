//! WebAuthn 凭证实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::*;

/// WebAuthn 凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnCredential {
    /// 凭证 ID
    pub id: WebAuthnCredentialId,

    /// 用户 ID
    pub user_id: Uuid,

    /// 租户 ID
    pub tenant_id: cuba_common::TenantId,

    /// WebAuthn 凭证 ID（二进制）
    pub credential_id: Vec<u8>,

    /// 公钥（二进制）
    pub public_key: Vec<u8>,

    /// 签名计数器
    pub counter: u32,

    /// 凭证名称
    pub name: String,

    /// 认证器 AAGUID
    pub aaguid: Option<Uuid>,

    /// 传输方式
    pub transports: Vec<String>,

    /// 是否可备份
    pub backup_eligible: bool,

    /// 是否已备份
    pub backup_state: bool,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 最后使用时间
    pub last_used_at: Option<DateTime<Utc>>,
}

impl WebAuthnCredential {
    /// 创建新凭证
    pub fn new(
        user_id: Uuid,
        credential_id: Vec<u8>,
        public_key: Vec<u8>,
        counter: u32,
        name: String,
        aaguid: Option<Uuid>,
        transports: Vec<String>,
        backup_eligible: bool,
        backup_state: bool,
        tenant_id: cuba_common::TenantId,
    ) -> Self {
        Self {
            id: WebAuthnCredentialId::new(),
            user_id,
            credential_id,
            public_key,
            counter,
            name,
            aaguid,
            transports,
            backup_eligible,
            backup_state,
            tenant_id,
            created_at: Utc::now(),
            last_used_at: None,
        }
    }

    /// 更新计数器
    pub fn update_counter(&mut self, new_counter: u32) {
        self.counter = new_counter;
        self.last_used_at = Some(Utc::now());
    }

    /// 转换为 webauthn-rs 的 Passkey 格式
    pub fn to_passkey(&self) -> Result<Passkey, WebAuthnCredentialError> {
        // 反序列化存储的 Passkey
        // 注意：我们在 from_passkey 中使用 serde_json 序列化了整个 Passkey
        serde_json::from_slice(&self.public_key)
            .map_err(|e| WebAuthnCredentialError::SerializationError(e.to_string()))
    }

    /// 从 webauthn-rs 的 Passkey 创建
    pub fn from_passkey(
        user_id: Uuid,
        name: String,
        passkey: &Passkey,
        aaguid: Option<Uuid>,
        transports: Vec<String>,
        tenant_id: cuba_common::TenantId,
    ) -> Result<Self, WebAuthnCredentialError> {
        // 使用方法访问器而不是字段
        let credential_id = passkey.cred_id().clone().into();

        // 序列化公钥
        let public_key = serde_json::to_vec(passkey.get_public_key())
            .map_err(|e| WebAuthnCredentialError::SerializationError(e.to_string()))?;

        Ok(Self::new(
            user_id,
            credential_id,
            public_key,
            0, // Initial counter
            name,
            aaguid,
            transports,
            false, // backup_eligible - will be updated on first auth
            false, // backup_state - will be updated on first auth
            tenant_id,
        ))
    }

    /// 从认证结果更新凭证
    pub fn update_from_authentication(&mut self, auth_result: &AuthenticationResult) {
        // 更新计数器
        if auth_result.counter() > self.counter {
            self.counter = auth_result.counter();
        }

        // 更新备份状态
        self.backup_state = auth_result.backup_state();
        if auth_result.backup_eligible() {
            self.backup_eligible = true;
        }

        // 更新最后使用时间
        self.last_used_at = Some(Utc::now());
    }

    /// 获取凭证 ID
    pub fn credential_id(&self) -> Vec<u8> {
        self.credential_id.clone()
    }
}

/// WebAuthn 凭证 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WebAuthnCredentialId(pub Uuid);

impl WebAuthnCredentialId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for WebAuthnCredentialId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WebAuthnCredentialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// WebAuthn 凭证错误
#[derive(Debug, thiserror::Error)]
pub enum WebAuthnCredentialError {
    #[error("Invalid credential ID")]
    InvalidCredentialId,

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Counter mismatch: expected > {expected}, got {actual}")]
    CounterMismatch { expected: u32, actual: u32 },
}

impl From<WebAuthnCredentialError> for cuba_errors::AppError {
    fn from(err: WebAuthnCredentialError) -> Self {
        cuba_errors::AppError::validation(err.to_string())
    }
}
