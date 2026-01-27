//! Refresh Token 实体

use chrono::{DateTime, Duration, Utc};
use cuba_common::{TenantId, UserId};
use serde::{Deserialize, Serialize};

use super::OAuthClientId;

/// Refresh Token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    /// Token 字符串（加密随机字符串）
    pub token: String,
    /// 关联的 Access Token
    pub access_token: String,
    /// Client ID
    pub client_id: OAuthClientId,
    /// 用户 ID
    pub user_id: UserId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 授权的 Scope 列表
    pub scopes: Vec<String>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 是否已撤销
    pub revoked: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl RefreshToken {
    /// 创建新的 Refresh Token
    pub fn new(
        token: String,
        access_token: String,
        client_id: OAuthClientId,
        user_id: UserId,
        tenant_id: TenantId,
        scopes: Vec<String>,
        lifetime_seconds: i64,
    ) -> Self {
        Self {
            token,
            access_token,
            client_id,
            user_id,
            tenant_id,
            scopes,
            expires_at: Utc::now() + Duration::seconds(lifetime_seconds),
            revoked: false,
            created_at: Utc::now(),
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 检查是否已撤销
    pub fn is_revoked(&self) -> bool {
        self.revoked
    }

    /// 检查是否有效（未过期且未撤销）
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked()
    }

    /// 撤销 Token
    pub fn revoke(&mut self) {
        self.revoked = true;
    }

    /// 获取剩余有效时间（秒）
    pub fn get_remaining_seconds(&self) -> i64 {
        let remaining = self.expires_at.timestamp() - Utc::now().timestamp();
        remaining.max(0)
    }
}

