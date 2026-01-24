//! Outbox trait 定义

use async_trait::async_trait;
use cuba_errors::AppResult;
use serde::{Deserialize, Serialize};

/// Outbox 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxMessage {
    pub id: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Outbox trait
#[async_trait]
pub trait OutboxPort: Send + Sync {
    /// 保存消息到 outbox
    async fn save(&self, message: &OutboxMessage) -> AppResult<()>;

    /// 获取未处理的消息
    async fn get_pending(&self, limit: usize) -> AppResult<Vec<OutboxMessage>>;

    /// 标记消息为已处理
    async fn mark_processed(&self, id: &str) -> AppResult<()>;

    /// 删除已处理的消息
    async fn delete_processed(&self, before: chrono::DateTime<chrono::Utc>) -> AppResult<u64>;
}
