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
        // 解析 credential_id
        let cred_id = CredentialID::try_from(self.credential_id.clone())
            .map_err(|_| WebAuthnCredentialError::InvalidCredentialId)?;

        // 解析公钥
        let cose_key = serde_cbor::from_slice(&self.public_key)
            .map_err(|e| WebAuthnCredentialError::InvalidPublicKey(e.to_string()))?;

        Ok(Passkey {
            cred_id,
            cred: cose_key,
            counter: self.counter,
            transports: None,
            user_verified: true,
            backup_eligible: self.backup_eligible,
            backup_state: self.backup_state,
            registration_policy: UserVerificationPolicy::Required,
            extensions: RegisteredExtensions::none(),
            attestation: ParsedAttestationData::None,
            attestation_format: AttestationFormat::None,
        })
    }

    /// 从 webauthn-rs 的 Passkey 创建
    pub fn from_passkey(
        user_id: Uuid,
        name: String,
        passkey: &Passkey,
        aaguid: Option<Uuid>,
        transports: Vec<String>,
    ) -> Result<Self, WebAuthnCredentialError> {
        let credential_id = passkey.cred_id.0.clone();
        let public_key = serde_cbor::to_vec(&passkey.cred)
            .map_err(|e| WebAuthnCredentialError::SerializationError(e.to_string()))?;

        Ok(Self::new(
            user_id,
            credential_id,
            public_key,
            passkey.counter,
            name,
            aaguid,
            transports,
            passkey.backup_eligible,
            passkey.backup_state,
        ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_webauthn_credential() {
        let user_id = Uuid::now_v7();
        let credential = WebAuthnCredential::new(
            user_id,
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            0,
            "YubiKey 5".to_string(),
            None,
            vec!["usb".to_string()],
            false,
            false,
        );

        assert_eq!(credential.user_id, user_id);
        assert_eq!(credential.name, "YubiKey 5");
        assert_eq!(credential.counter, 0);
        assert!(credential.last_used_at.is_none());
    }

    #[test]
    fn test_update_counter() {
        let mut credential = WebAuthnCredential::new(
            Uuid::now_v7(),
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            0,
            "Test".to_string(),
            None,
            vec![],
            false,
            false,
        );

        credential.update_counter(5);
        assert_eq!(credential.counter, 5);
        assert!(credential.last_used_at.is_some());
    }
}
