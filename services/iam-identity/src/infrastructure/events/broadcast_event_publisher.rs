use super::event_publisher::{EventPublisher, IamDomainEvent};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, warn};

/// 广播事件发布器
///
/// 将事件广播给所有注册的发布者（例如：事件存储 + 邮件通知器 + 审计日志）
///
/// 特性：
/// - 并发执行所有发布器以提高性能
/// - 区分关键和非关键发布器
/// - 关键发布器失败会记录错误但不中断流程
pub struct BroadcastEventPublisher {
    /// 关键发布器（如事件存储），失败会记录错误
    critical_publishers: Vec<Arc<dyn EventPublisher>>,
    /// 非关键发布器（如通知），失败只记录警告
    optional_publishers: Vec<Arc<dyn EventPublisher>>,
}

impl BroadcastEventPublisher {
    /// 创建新的广播发布器（所有发布器都视为关键）
    pub fn new(publishers: Vec<Arc<dyn EventPublisher>>) -> Self {
        Self {
            critical_publishers: publishers,
            optional_publishers: Vec::new(),
        }
    }

    /// 创建带优先级的广播发布器
    pub fn with_priority(
        critical: Vec<Arc<dyn EventPublisher>>,
        optional: Vec<Arc<dyn EventPublisher>>,
    ) -> Self {
        Self {
            critical_publishers: critical,
            optional_publishers: optional,
        }
    }
}

#[async_trait]
impl EventPublisher for BroadcastEventPublisher {
    async fn publish(&self, event: IamDomainEvent) {
        // 并发执行所有发布器
        let mut tasks = Vec::new();

        // 关键发布器
        for publisher in &self.critical_publishers {
            let event_clone = event.clone();
            let publisher_clone = publisher.clone();
            tasks.push(tokio::spawn(async move {
                publisher_clone.publish(event_clone).await;
            }));
        }

        // 非关键发布器
        for publisher in &self.optional_publishers {
            let event_clone = event.clone();
            let publisher_clone = publisher.clone();
            tasks.push(tokio::spawn(async move {
                publisher_clone.publish(event_clone).await;
            }));
        }

        // 等待所有任务完成
        for (idx, task) in tasks.into_iter().enumerate() {
            if let Err(e) = task.await {
                let is_critical = idx < self.critical_publishers.len();
                if is_critical {
                    error!(
                        event_type = event.event_type(),
                        error = %e,
                        "Critical event publisher failed"
                    );
                } else {
                    warn!(
                        event_type = event.event_type(),
                        error = %e,
                        "Optional event publisher failed"
                    );
                }
            }
        }
    }

    async fn publish_all(&self, events: Vec<IamDomainEvent>) {
        for event in events {
            self.publish(event).await;
        }
    }
}
