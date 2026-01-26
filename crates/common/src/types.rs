//! 通用类型定义

use chrono::{DateTime, Utc};
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 租户 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From)]
#[display("{_0}")]
pub struct TenantId(pub Uuid);

impl TenantId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
    
    pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
        Self::from_string(s)
    }
}

impl Default for TenantId {
    fn default() -> Self {
        Self::new()
    }
}

/// 用户 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From)]
#[display("{_0}")]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

/// 审计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditInfo {
    pub created_at: DateTime<Utc>,
    pub created_by: Option<UserId>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<UserId>,
}

impl AuditInfo {
    pub fn new(user_id: Option<UserId>) -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            created_by: user_id.clone(),
            updated_at: now,
            updated_by: user_id,
        }
    }

    pub fn update(&mut self, user_id: Option<UserId>) {
        self.updated_at = Utc::now();
        self.updated_by = user_id;
    }
}

impl Default for AuditInfo {
    fn default() -> Self {
        Self::new(None)
    }
}

/// 分页参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

impl Pagination {
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.page_size
    }
}

/// 分页结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

impl<T> PagedResult<T> {
    pub fn new(items: Vec<T>, total: u64, pagination: &Pagination) -> Self {
        Self {
            items,
            total,
            page: pagination.page,
            page_size: pagination.page_size,
        }
    }

    pub fn total_pages(&self) -> u32 {
        ((self.total as f64) / (self.page_size as f64)).ceil() as u32
    }
}
