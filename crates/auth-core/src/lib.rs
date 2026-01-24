//! cuba-auth-core - 认证核心库
//!
//! JWT/Claims/RBAC 核心逻辑

use chrono::{Duration, Utc};
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Expiration time
    pub exp: i64,
    /// Issued at
    pub iat: i64,
    /// JWT ID
    pub jti: String,
    /// Permissions
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Roles
    #[serde(default)]
    pub roles: Vec<String>,
}

impl Claims {
    pub fn new(
        user_id: &UserId,
        tenant_id: &TenantId,
        permissions: Vec<String>,
        roles: Vec<String>,
        expires_in_secs: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id.0.to_string(),
            tenant_id: tenant_id.0.to_string(),
            exp: (now + Duration::seconds(expires_in_secs)).timestamp(),
            iat: now.timestamp(),
            jti: Uuid::now_v7().to_string(),
            permissions,
            roles,
        }
    }

    pub fn user_id(&self) -> AppResult<UserId> {
        Uuid::parse_str(&self.sub)
            .map(UserId::from_uuid)
            .map_err(|_| AppError::unauthorized("Invalid user ID in token"))
    }

    pub fn tenant_id(&self) -> AppResult<TenantId> {
        Uuid::parse_str(&self.tenant_id)
            .map(TenantId::from_uuid)
            .map_err(|_| AppError::unauthorized("Invalid tenant ID in token"))
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    pub fn has_any_permission(&self, permissions: &[&str]) -> bool {
        permissions.iter().any(|p| self.has_permission(p))
    }

    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions.iter().all(|p| self.has_permission(p))
    }
}

/// Token 服务
pub struct TokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expires_in: i64,
    refresh_token_expires_in: i64,
}

impl TokenService {
    pub fn new(secret: &str, access_token_expires_in: i64, refresh_token_expires_in: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_token_expires_in,
            refresh_token_expires_in,
        }
    }

    /// 生成访问令牌
    pub fn generate_access_token(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        permissions: Vec<String>,
        roles: Vec<String>,
    ) -> AppResult<String> {
        let claims = Claims::new(
            user_id,
            tenant_id,
            permissions,
            roles,
            self.access_token_expires_in,
        );

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::internal(format!("Failed to generate token: {}", e)))
    }

    /// 生成刷新令牌
    pub fn generate_refresh_token(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<String> {
        let claims = Claims::new(
            user_id,
            tenant_id,
            vec![],
            vec![],
            self.refresh_token_expires_in,
        );

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::internal(format!("Failed to generate refresh token: {}", e)))
    }

    /// 验证令牌
    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|e| AppError::unauthorized(format!("Invalid token: {}", e)))
    }

    /// 获取访问令牌过期时间（秒）
    pub fn access_token_expires_in(&self) -> i64 {
        self.access_token_expires_in
    }
}

/// 权限检查宏
#[macro_export]
macro_rules! require_permission {
    ($claims:expr, $permission:expr) => {
        if !$claims.has_permission($permission) {
            return Err(cuba_errors::AppError::forbidden(format!(
                "Missing permission: {}",
                $permission
            )));
        }
    };
}

/// 角色检查宏
#[macro_export]
macro_rules! require_role {
    ($claims:expr, $role:expr) => {
        if !$claims.has_role($role) {
            return Err(cuba_errors::AppError::forbidden(format!(
                "Missing role: {}",
                $role
            )));
        }
    };
}
