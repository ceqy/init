use std::sync::Arc;
use async_trait::async_trait;
use super::event_publisher::{EventPublisher, IamDomainEvent};
/// 广播事件发布器
///
/// 将事件广播给所有注册的发布者（例如：事件存储 + 邮件通知器 + 审计日志）
pub struct BroadcastEventPublisher {
    publishers: Vec<Arc<dyn EventPublisher>>,
}

impl BroadcastEventPublisher {
    pub fn new(publishers: Vec<Arc<dyn EventPublisher>>) -> Self {
        Self { publishers }
    }
}

#[async_trait]
impl EventPublisher for BroadcastEventPublisher {
    async fn publish(&self, event: IamDomainEvent) {
        // 创建多个 future 并发执行，或者按顺序执行。这里选择顺序执行以确保一致性，
        // 但为了性能也可以考虑 tokio::join!
        for publisher in &self.publishers {
            // 注意：因为 publish 接受所有权，我们需要 clone event。
            // 之前的 trait 定义 `publish(&self, event: IamDomainEvent)` 消耗了 event。
            // 这意味着 Broadcast 需要 clone event 给每个 subscriber。
            publisher.publish(event.clone()).await;
        }
    }

    async fn publish_all(&self, events: Vec<IamDomainEvent>) {
        for event in events {
            self.publish(event).await;
        }
    }
}
