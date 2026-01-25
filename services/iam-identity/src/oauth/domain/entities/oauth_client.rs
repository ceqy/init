//! OAuth Client 实体

pub mod access_token;
pub mod authorization_code;
pub mod refresh_token;

pub use access_token::AccessToken;
pub use authorization_code::AuthorizationCode;
pub use refresh_token::RefreshToken;

use chrono::{DateTime, Utc};
use cuba_common::{TenantId, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// OAuth Client ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OAuthClientId(pub Uuid);

impl OAuthClientId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for OAuthClientId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for OAuthClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// OAuth Client 类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OAuthClientType {
    /// 机密客户端（有 client_secret）
    Confidential,
    /// 公开客户端（无 client_secret，如 SPA、移动应用）
    Public,
}

/// OAuth 授权类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrantType {
    AuthorizationCode,
    ClientCredentials,
    RefreshToken,
    Implicit,
    Password,
}

impl std::fmt::Display for GrantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthorizationCode => write!(f, "authorization_code"),
            Self::ClientCredentials => write!(f, "client_credentials"),
            Self::RefreshToken => write!(f, "refresh_token"),
            Self::Implicit => write!(f, "implicit"),
            Self::Password => write!(f, "password"),
        }
    }
}

/// OAuth Client 实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClient {
    /// Client ID
    pub id: OAuthClientId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 创建者 ID
    pub owner_id: UserId,
    /// Client 名称
    pub name: String,
    /// Client 描述
    pub description: Option<String>,
    /// Client Secret（哈希后）
    pub client_secret_hash: Option<String>,
    /// Client 类型
    pub client_type: OAuthClientType,
    /// 允许的授权类型
    pub grant_types: Vec<GrantType>,
    /// 重定向 URI 列表
    pub redirect_uris: Vec<String>,
    /// 允许的 Scope 列表
    pub allowed_scopes: Vec<String>,
    /// Access Token 有效期（秒）
    pub access_token_lifetime: i64,
    /// Refresh Token 有效期（秒）
    pub refresh_token_lifetime: i64,
    /// 是否需要 PKCE
    pub require_pkce: bool,
    /// 是否需要用户同意
    pub require_consent: bool,
    /// 是否激活
    pub is_active: bool,
    /// Logo URL
    pub logo_url: Option<String>,
    /// 主页 URL
    pub homepage_url: Option<String>,
    /// 隐私政策 URL
    pub privacy_policy_url: Option<String>,
    /// 服务条款 URL
    pub terms_of_service_url: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl OAuthClient {
    /// 创建新的 OAuth Client
    pub fn new(
        tenant_id: TenantId,
        owner_id: UserId,
        name: String,
        client_type: OAuthClientType,
        redirect_uris: Vec<String>,
    ) -> Result<Self, OAuthClientError> {
        // 验证名称
        if name.is_empty() {
            return Err(OAuthClientError::Validation("Client name cannot be empty".to_string()));
        }

        // 验证重定向 URI
        if redirect_uris.is_empty() {
            return Err(OAuthClientError::Validation("At least one redirect URI is required".to_string()));
        }

        for uri in &redirect_uris {
            Self::validate_redirect_uri(uri)?;
        }

        Ok(Self {
            id: OAuthClientId::new(),
            tenant_id,
            owner_id,
            name,
            description: None,
            client_secret_hash: None,
            client_type,
            grant_types: vec![GrantType::AuthorizationCode],
            redirect_uris,
            allowed_scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
            access_token_lifetime: 3600,      // 1 hour
            refresh_token_lifetime: 2592000,  // 30 days
            require_pkce: true,
            require_consent: true,
            is_active: true,
            logo_url: None,
            homepage_url: None,
            privacy_policy_url: None,
            terms_of_service_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// 验证重定向 URI
    fn validate_redirect_uri(uri: &str) -> Result<(), OAuthClientError> {
        // 必须是 HTTPS（开发环境可以是 HTTP localhost）
        if !uri.starts_with("https://") 
            && !uri.starts_with("http://localhost")
            && !uri.starts_with("http://127.0.0.1") {
            return Err(OAuthClientError::Validation(
                "Redirect URI must use HTTPS or be localhost".to_string()
            ));
        }

        // 不能包含 fragment
        if uri.contains('#') {
            return Err(OAuthClientError::Validation(
                "Redirect URI cannot contain fragment".to_string()
            ));
        }

        Ok(())
    }

    /// 设置 Client Secret
    pub fn set_client_secret(&mut self, secret_hash: String) {
        self.client_secret_hash = Some(secret_hash);
        self.updated_at = Utc::now();
    }

    /// 轮换 Client Secret
    pub fn rotate_client_secret(&mut self, new_secret_hash: String) {
        self.client_secret_hash = Some(new_secret_hash);
        self.updated_at = Utc::now();
    }

    /// 验证 Client Secret
    pub fn verify_client_secret(&self, secret: &str) -> Result<bool, OAuthClientError> {
        match &self.client_secret_hash {
            Some(hash) => {
                // 这里应该使用密码哈希验证
                // 简化实现，实际应使用 bcrypt 或 argon2
                Ok(hash == secret)
            }
            None => Ok(false),
        }
    }

    /// 验证重定向 URI 是否在允许列表中
    pub fn validate_redirect_uri_match(&self, uri: &str) -> bool {
        self.redirect_uris.iter().any(|allowed| allowed == uri)
    }

    /// 验证授权类型是否允许
    pub fn is_grant_type_allowed(&self, grant_type: &GrantType) -> bool {
        self.grant_types.contains(grant_type)
    }

    /// 验证 Scope 是否允许
    pub fn validate_scopes(&self, requested_scopes: &[String]) -> bool {
        requested_scopes.iter().all(|scope| self.allowed_scopes.contains(scope))
    }

    /// 更新 Client 信息
    pub fn update(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        redirect_uris: Option<Vec<String>>,
        allowed_scopes: Option<Vec<String>>,
    ) -> Result<(), OAuthClientError> {
        if let Some(n) = name {
            if n.is_empty() {
                return Err(OAuthClientError::Validation("Client name cannot be empty".to_string()));
            }
            self.name = n;
        }

        if let Some(d) = description {
            self.description = Some(d);
        }

        if let Some(uris) = redirect_uris {
            if uris.is_empty() {
                return Err(OAuthClientError::Validation("At least one redirect URI is required".to_string()));
            }
            for uri in &uris {
                Self::validate_redirect_uri(uri)?;
            }
            self.redirect_uris = uris;
        }

        if let Some(scopes) = allowed_scopes {
            self.allowed_scopes = scopes;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// 激活 Client
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// 停用 Client
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
}

/// OAuth Client 错误
#[derive(Debug, thiserror::Error)]
pub enum OAuthClientError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Client is not active")]
    NotActive,

    #[error("Invalid client secret")]
    InvalidSecret,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_oauth_client() {
        let tenant_id = TenantId::new();
        let owner_id = UserId::new();
        let result = OAuthClient::new(
            tenant_id,
            owner_id,
            "Test App".to_string(),
            OAuthClientType::Confidential,
            vec!["https://example.com/callback".to_string()],
        );

        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.name, "Test App");
        assert!(client.is_active);
        assert!(client.require_pkce);
    }

    #[test]
    fn test_validate_redirect_uri() {
        assert!(OAuthClient::validate_redirect_uri("https://example.com/callback").is_ok());
        assert!(OAuthClient::validate_redirect_uri("http://localhost:3000/callback").is_ok());
        assert!(OAuthClient::validate_redirect_uri("http://example.com/callback").is_err());
        assert!(OAuthClient::validate_redirect_uri("https://example.com/callback#fragment").is_err());
    }

    #[test]
    fn test_validate_redirect_uri_match() {
        let tenant_id = TenantId::new();
        let owner_id = UserId::new();
        let client = OAuthClient::new(
            tenant_id,
            owner_id,
            "Test App".to_string(),
            OAuthClientType::Confidential,
            vec!["https://example.com/callback".to_string()],
        ).unwrap();

        assert!(client.validate_redirect_uri_match("https://example.com/callback"));
        assert!(!client.validate_redirect_uri_match("https://evil.com/callback"));
    }

    #[test]
    fn test_validate_scopes() {
        let tenant_id = TenantId::new();
        let owner_id = UserId::new();
        let client = OAuthClient::new(
            tenant_id,
            owner_id,
            "Test App".to_string(),
            OAuthClientType::Confidential,
            vec!["https://example.com/callback".to_string()],
        ).unwrap();

        assert!(client.validate_scopes(&["openid".to_string(), "profile".to_string()]));
        assert!(!client.validate_scopes(&["admin".to_string()]));
    }
}
