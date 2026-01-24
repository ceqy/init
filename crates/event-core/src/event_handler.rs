//! Event Handler 定义

use async_trait::async_trait;
use cuba_errors::AppResult;

use crate::DomainEvent;

/// Event Handler trait
#[async_trait]
pub trait EventHandler<E: DomainEvent>: Send + Sync {
    async fn handle(&self, event: &E) -> AppResult<()>;
}

/// Event Subscriber trait
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// 订阅的事件类型
    fn event_types(&self) -> Vec<&'static str>;

    /// 处理事件
    async fn handle(&self, event_type: &str, payload: &str) -> AppResult<()>;
}
