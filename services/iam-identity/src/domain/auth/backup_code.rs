//! 备份码实体

use chrono::{DateTime, Utc};
use common::UserId;
use domain_core::Entity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 备份码 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackupCodeId(pub Uuid);

impl BackupCodeId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for BackupCodeId {
    fn default() -> Self {
        Self::new()
    }
}

/// 备份码实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCode {
    pub id: BackupCodeId,
    pub user_id: UserId,
    pub tenant_id: common::TenantId,
    pub code_hash: String,
    pub used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl BackupCode {
    pub fn new(user_id: UserId, tenant_id: common::TenantId, code_hash: String) -> Self {
        Self {
            id: BackupCodeId::new(),
            user_id,
            tenant_id,
            code_hash,
            used: false,
            used_at: None,
            created_at: Utc::now(),
        }
    }

    /// 标记为已使用
    pub fn mark_as_used(&mut self) {
        self.used = true;
        self.used_at = Some(Utc::now());
    }

    /// 是否可用
    pub fn is_available(&self) -> bool {
        !self.used
    }
}

impl Entity for BackupCode {
    type Id = BackupCodeId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
