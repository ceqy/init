//! 密码重置令牌实体

use chrono::{DateTime, Duration, Utc};
use cuba_common::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 密码重置令牌 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PasswordResetTokenId(pub Uuid);

impl PasswordResetTokenId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for PasswordResetTokenId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PasswordResetTokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 密码重置令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    /// 令牌 ID
    pub id: PasswordResetTokenId,

    /// 用户 ID
    pub user_id: UserId,

    /// 租户 ID
    pub tenant_id: cuba_common::TenantId,

    /// 令牌哈希（存储 SHA256 哈希，不存储原始令牌）
    pub token_hash: String,

    /// 过期时间
    pub expires_at: DateTime<Utc>,

    /// 是否已使用
    pub used: bool,

    /// 使用时间
    pub used_at: Option<DateTime<Utc>>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl PasswordResetToken {
    /// 创建新的密码重置令牌
    ///
    /// # 参数
    /// - `user_id`: 用户 ID
    /// - `tenant_id`: 租户 ID
    /// - `token_hash`: 令牌的 SHA256 哈希
    /// - `expires_in_minutes`: 过期时间（分钟）
    pub fn new(
        user_id: UserId,
        tenant_id: cuba_common::TenantId,
        token_hash: String,
        expires_in_minutes: i64,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::minutes(expires_in_minutes);

        Self {
            id: PasswordResetTokenId::new(),
            user_id,
            tenant_id,
            token_hash,
            expires_at,
            used: false,
            used_at: None,
            created_at: now,
        }
    }

    /// 检查令牌是否有效
    pub fn is_valid(&self) -> bool {
        !self.used && !self.is_expired()
    }

    /// 检查令牌是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 标记令牌为已使用
    pub fn mark_as_used(&mut self) {
        self.used = true;
        self.used_at = Some(Utc::now());
    }

    /// 获取剩余有效时间（秒）
    pub fn remaining_seconds(&self) -> i64 {
        let now = Utc::now();
        if now > self.expires_at {
            0
        } else {
            (self.expires_at - now).num_seconds()
        }
    }
}
