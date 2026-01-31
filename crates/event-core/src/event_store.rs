//! Event Store trait 定义

use async_trait::async_trait;
use errors::AppResult;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::EventEnvelope;

/// 存储的事件记录
#[derive(Debug, Clone)]
pub struct StoredEvent {
    pub id: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub version: u64,
    pub payload: String,
    pub metadata: String,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Event Store trait
#[async_trait]
pub trait EventStore: Send + Sync {
    /// 追加事件
    async fn append<E: Serialize + Send + Sync>(
        &self,
        envelope: &EventEnvelope<E>,
    ) -> AppResult<()>;

    /// 获取聚合的所有事件
    async fn get_events(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> AppResult<Vec<StoredEvent>>;

    /// 获取聚合从指定版本开始的事件
    async fn get_events_from_version(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
        from_version: u64,
    ) -> AppResult<Vec<StoredEvent>>;

    /// 获取聚合的当前版本
    async fn get_current_version(&self, aggregate_type: &str, aggregate_id: &str)
    -> AppResult<u64>;
}

/// 从存储的事件反序列化
pub fn deserialize_event<E: DeserializeOwned>(stored: &StoredEvent) -> AppResult<E> {
    serde_json::from_str(&stored.payload)
        .map_err(|e| errors::AppError::internal(format!("Failed to deserialize event: {}", e)))
}
