//! 会话实体

use chrono::{DateTime, Utc};
use cuba_common::UserId;
use cuba_domain_core::Entity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 会话 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// 会话实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub tenant_id: cuba_common::TenantId,
    pub refresh_token_hash: String,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub revoked: bool,
}

impl Session {
    pub fn new(
        user_id: UserId,
        tenant_id: cuba_common::TenantId,
        refresh_token_hash: String,
        expires_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            user_id,
            tenant_id,
            refresh_token_hash,
            device_info: None,
            ip_address: None,
            user_agent: None,
            created_at: now,
            expires_at,
            last_activity_at: now,
            revoked: false,
        }
    }

    pub fn with_device_info(mut self, device_info: impl Into<String>) -> Self {
        self.device_info = Some(device_info.into());
        self
    }

    pub fn with_ip_address(mut self, ip_address: impl Into<String>) -> Self {
        self.ip_address = Some(ip_address.into());
        self
    }

    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked && self.expires_at > Utc::now()
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }

    pub fn revoke(&mut self) {
        self.revoked = true;
    }

    pub fn update_activity(&mut self) {
        self.last_activity_at = Utc::now();
    }

    pub fn refresh(&mut self, new_token_hash: String, new_expires_at: DateTime<Utc>) {
        self.refresh_token_hash = new_token_hash;
        self.expires_at = new_expires_at;
        self.last_activity_at = Utc::now();
    }
}

impl Entity for Session {
    type Id = SessionId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
