//! Domain Event 定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Domain Event trait
pub trait DomainEvent: Send + Sync + Serialize {
    /// 事件类型名称
    fn event_type(&self) -> &'static str;

    /// 聚合类型
    fn aggregate_type(&self) -> &'static str;

    /// 聚合 ID
    fn aggregate_id(&self) -> String;
}

/// 事件信封（包含元数据）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<E> {
    /// 事件 ID
    pub id: Uuid,
    /// 事件类型
    pub event_type: String,
    /// 聚合类型
    pub aggregate_type: String,
    /// 聚合 ID
    pub aggregate_id: String,
    /// 事件版本
    pub version: u64,
    /// 事件数据
    pub data: E,
    /// 元数据
    pub metadata: EventMetadata,
    /// 发生时间
    pub occurred_at: DateTime<Utc>,
}

impl<E: DomainEvent> EventEnvelope<E> {
    pub fn new(event: E, version: u64, metadata: EventMetadata) -> Self {
        Self {
            id: Uuid::now_v7(),
            event_type: event.event_type().to_string(),
            aggregate_type: event.aggregate_type().to_string(),
            aggregate_id: event.aggregate_id(),
            version,
            data: event,
            metadata,
            occurred_at: Utc::now(),
        }
    }
}

/// 事件元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    /// 触发用户 ID
    pub user_id: Option<String>,
    /// 租户 ID
    pub tenant_id: Option<String>,
    /// 关联 ID（用于追踪）
    pub correlation_id: Option<String>,
    /// 因果 ID
    pub causation_id: Option<String>,
}

impl EventMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn with_correlation(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
}
