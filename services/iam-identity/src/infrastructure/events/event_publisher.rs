//! 事件发布器
//!
//! 提供领域事件的发布功能

use async_trait::async_trait;
use cuba_errors::AppResult;
use cuba_event_core::DomainEvent;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 事件发布器 trait
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// 发布单个事件
    async fn publish<E: DomainEvent + Send + Sync + 'static>(&self, event: E) -> AppResult<()>;
    
    /// 批量发布事件
    async fn publish_all<E: DomainEvent + Send + Sync + 'static>(&self, events: Vec<E>) -> AppResult<()>;
}

/// 事件订阅者 trait
#[async_trait]
pub trait EventSubscriber<E: DomainEvent>: Send + Sync {
    /// 处理事件
    async fn handle(&self, event: &E) -> AppResult<()>;
}

/// 内存事件总线实现
/// 
/// 用于开发和测试环境
pub struct InMemoryEventBus {
    handlers: Arc<RwLock<Vec<Box<dyn Fn(&dyn std::any::Any) + Send + Sync>>>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventBus {
    async fn publish<E: DomainEvent + Send + Sync + 'static>(&self, event: E) -> AppResult<()> {
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            handler(&event);
        }
        
        tracing::info!(
            event_type = event.event_type(),
            aggregate_type = event.aggregate_type(),
            aggregate_id = event.aggregate_id(),
            "Domain event published"
        );
        
        Ok(())
    }
    
    async fn publish_all<E: DomainEvent + Send + Sync + 'static>(&self, events: Vec<E>) -> AppResult<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }
}

/// NoOp 事件发布器
/// 
/// 不执行任何操作，用于禁用事件发布
pub struct NoOpEventPublisher;

#[async_trait]
impl EventPublisher for NoOpEventPublisher {
    async fn publish<E: DomainEvent + Send + Sync + 'static>(&self, _event: E) -> AppResult<()> {
        Ok(())
    }
    
    async fn publish_all<E: DomainEvent + Send + Sync + 'static>(&self, _events: Vec<E>) -> AppResult<()> {
        Ok(())
    }
}

/// 日志事件发布器
/// 
/// 仅记录事件日志，不执行其他操作
pub struct LoggingEventPublisher;

#[async_trait]
impl EventPublisher for LoggingEventPublisher {
    async fn publish<E: DomainEvent + Send + Sync + 'static>(&self, event: E) -> AppResult<()> {
        tracing::info!(
            event_type = event.event_type(),
            aggregate_type = event.aggregate_type(),
            aggregate_id = event.aggregate_id(),
            "Domain event: {}",
            event.event_type()
        );
        Ok(())
    }
    
    async fn publish_all<E: DomainEvent + Send + Sync + 'static>(&self, events: Vec<E>) -> AppResult<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }
}
