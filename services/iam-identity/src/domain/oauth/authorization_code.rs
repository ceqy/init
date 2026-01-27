//! 授权码实体

use chrono::{DateTime, Duration, Utc};
use cuba_common::{TenantId, UserId};
use serde::{Deserialize, Serialize};

use super::{OAuthClientId, OAuthClientError};

/// 授权码
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCode {
    /// 授权码（加密随机字符串）
    pub code: String,
    /// Client ID
    pub client_id: OAuthClientId,
    /// 用户 ID
    pub user_id: UserId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 重定向 URI
    pub redirect_uri: String,
    /// 授权的 Scope 列表
    pub scopes: Vec<String>,
    /// PKCE code_challenge
    pub code_challenge: Option<String>,
    /// PKCE code_challenge_method (S256 or plain)
    pub code_challenge_method: Option<String>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 是否已使用
    pub used: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl AuthorizationCode {
    /// 创建新的授权码
    pub fn new(
        code: String,
        client_id: OAuthClientId,
        user_id: UserId,
        tenant_id: TenantId,
        redirect_uri: String,
        scopes: Vec<String>,
        code_challenge: Option<String>,
        code_challenge_method: Option<String>,
    ) -> Self {
        Self {
            code,
            client_id,
            user_id,
            tenant_id,
            redirect_uri,
            scopes,
            code_challenge,
            code_challenge_method,
            expires_at: Utc::now() + Duration::minutes(10), // 10分钟过期
            used: false,
            created_at: Utc::now(),
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 检查是否已使用
    pub fn is_used(&self) -> bool {
        self.used
    }

    /// 标记为已使用
    pub fn mark_as_used(&mut self) {
        self.used = true;
    }

    /// 验证 PKCE code_verifier
    pub fn verify_code_verifier(&self, code_verifier: &str) -> Result<bool, OAuthClientError> {
        match (&self.code_challenge, &self.code_challenge_method) {
            (Some(challenge), Some(method)) => {
                let computed_challenge = match method.as_str() {
                    "S256" => {
                        use base64::Engine;
                        use sha2::{Digest, Sha256};
                        let hash = Sha256::digest(code_verifier.as_bytes());
                        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&hash)
                    }
                    "plain" => code_verifier.to_string(),
                    _ => {
                        return Err(OAuthClientError::Validation(
                            "Invalid code_challenge_method".to_string(),
                        ))
                    }
                };

                Ok(computed_challenge == *challenge)
            }
            (None, None) => Ok(true), // 没有 PKCE 要求
            _ => Err(OAuthClientError::Validation(
                "Invalid PKCE configuration".to_string(),
            )),
        }
    }
}

