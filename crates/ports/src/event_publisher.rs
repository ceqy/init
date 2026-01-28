//! Event Publisher trait 定义

use async_trait::async_trait;
use cuba_errors::AppResult;
use serde::Serialize;

/// 事件发布者 trait
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// 发布事件
    async fn publish<E: Serialize + Send + Sync>(&self, topic: &str, event: &E) -> AppResult<()>;

    /// 发布原始 JSON 字符串 (用于 Outbox 模式)
    async fn publish_raw(&self, topic: &str, payload: &str) -> AppResult<()>;

    /// 批量发布事件
    async fn publish_batch<E: Serialize + Send + Sync>(
        &self,
        topic: &str,
        events: &[E],
    ) -> AppResult<()>;
}

/// 事件订阅者 trait
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// 订阅事件
    async fn subscribe(&self, topic: &str, group: &str) -> AppResult<()>;

    /// 取消订阅
    async fn unsubscribe(&self, topic: &str) -> AppResult<()>;
}
