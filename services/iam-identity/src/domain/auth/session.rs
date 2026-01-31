//! 会话实体

use chrono::{DateTime, Utc};
use common::UserId;
use domain_core::Entity;
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
    pub tenant_id: common::TenantId,
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
        tenant_id: common::TenantId,
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

// ============================================================
// 单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_session() -> Session {
        let user_id = UserId::new();
        let tenant_id = common::TenantId::new();
        let refresh_token_hash = "test_hash".to_string();
        let expires_at = Utc::now() + chrono::Duration::hours(24);

        Session::new(user_id, tenant_id, refresh_token_hash, expires_at)
    }

    #[test]
    fn test_create_session() {
        let session = create_test_session();

        assert!(!session.revoked);
        assert!(session.device_info.is_none());
        assert!(session.ip_address.is_none());
        assert!(session.user_agent.is_none());
        assert!(session.is_valid());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_with_device_info() {
        let session = create_test_session().with_device_info("iPhone 13");

        assert_eq!(session.device_info, Some("iPhone 13".to_string()));
    }

    #[test]
    fn test_session_with_ip_address() {
        let session = create_test_session().with_ip_address("192.168.1.1");

        assert_eq!(session.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_session_with_user_agent() {
        let session = create_test_session().with_user_agent("Mozilla/5.0");

        assert_eq!(session.user_agent, Some("Mozilla/5.0".to_string()));
    }

    #[test]
    fn test_session_builder_pattern() {
        let session = create_test_session()
            .with_device_info("iPhone 13")
            .with_ip_address("192.168.1.1")
            .with_user_agent("Mozilla/5.0");

        assert!(session.device_info.is_some());
        assert!(session.ip_address.is_some());
        assert!(session.user_agent.is_some());
    }

    #[test]
    fn test_is_valid() {
        let session = create_test_session();
        assert!(session.is_valid());
    }

    #[test]
    fn test_is_expired() {
        let user_id = UserId::new();
        let tenant_id = common::TenantId::new();
        let refresh_token_hash = "test_hash".to_string();
        let expires_at = Utc::now() - chrono::Duration::hours(1); // 已过期

        let session = Session::new(user_id, tenant_id, refresh_token_hash, expires_at);

        assert!(session.is_expired());
        assert!(!session.is_valid());
    }

    #[test]
    fn test_revoke_session() {
        let mut session = create_test_session();

        session.revoke();

        assert!(session.revoked);
        assert!(!session.is_valid());
    }

    #[test]
    fn test_update_activity() {
        let mut session = create_test_session();
        let old_activity_time = session.last_activity_at;

        // 等待一小段时间
        std::thread::sleep(std::time::Duration::from_millis(10));

        session.update_activity();

        assert!(session.last_activity_at > old_activity_time);
    }

    #[test]
    fn test_refresh_session() {
        let mut session = create_test_session();
        let old_token_hash = session.refresh_token_hash.clone();
        let old_expires_at = session.expires_at;

        let new_token_hash = "new_hash".to_string();
        let new_expires_at = Utc::now() + chrono::Duration::hours(48);

        session.refresh(new_token_hash.clone(), new_expires_at);

        assert_ne!(session.refresh_token_hash, old_token_hash);
        assert_eq!(session.refresh_token_hash, new_token_hash);
        assert!(session.expires_at > old_expires_at);
    }
}
