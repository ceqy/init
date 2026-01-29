//! Outbox 发布器 - 后台任务异步发布事件
//!
//! 定期从 outbox 表读取待发布事件并发送到消息队列

use cuba_errors::AppResult;
use cuba_ports::EventPublisher;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use super::outbox_repository::{OutboxRepository, PostgresOutboxRepository};

/// Outbox 发布器
pub struct OutboxPublisher<EP: EventPublisher> {
    outbox_repo: Arc<PostgresOutboxRepository>,
    event_publisher: Arc<EP>,
    batch_size: i64,
    poll_interval: Duration,
}

impl<EP: EventPublisher + 'static> OutboxPublisher<EP> {
    pub fn new(outbox_repo: Arc<PostgresOutboxRepository>, event_publisher: Arc<EP>) -> Self {
        Self {
            outbox_repo,
            event_publisher,
            batch_size: 100,
            poll_interval: Duration::from_secs(1),
        }
    }

    pub fn with_batch_size(mut self, size: i64) -> Self {
        self.batch_size = size;
        self
    }

    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// 启动后台发布循环
    pub async fn start(self) {
        info!(
            "Starting outbox publisher with interval {:?}",
            self.poll_interval
        );

        let mut interval = interval(self.poll_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.publish_pending().await {
                error!("Error publishing outbox events: {:?}", e);
            }
        }
    }

    /// 发布待处理的事件
    async fn publish_pending(&self) -> AppResult<()> {
        let events = self.outbox_repo.get_pending(self.batch_size).await?;

        if events.is_empty() {
            return Ok(());
        }

        info!("Publishing {} outbox events", events.len());

        for event in events {
            let channel = format!(
                "rbac.{}.{}",
                event.aggregate_type.to_lowercase(),
                event.event_type.to_lowercase()
            );

            // 尝试发布
            match self
                .event_publisher
                .publish_raw(&channel, &event.payload)
                .await
            {
                Ok(_) => {
                    self.outbox_repo.mark_published(event.id).await?;
                }
                Err(e) => {
                    warn!("Failed to publish event {}: {:?}", event.id, e);
                    self.outbox_repo
                        .mark_failed(event.id, &e.to_string())
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// 单次处理（用于测试或手动触发）
    pub async fn process_once(&self) -> AppResult<usize> {
        let events = self.outbox_repo.get_pending(self.batch_size).await?;
        let count = events.len();

        for event in events {
            let channel = format!(
                "rbac.{}.{}",
                event.aggregate_type.to_lowercase(),
                event.event_type.to_lowercase()
            );

            match self
                .event_publisher
                .publish_raw(&channel, &event.payload)
                .await
            {
                Ok(_) => {
                    self.outbox_repo.mark_published(event.id).await?;
                }
                Err(e) => {
                    self.outbox_repo
                        .mark_failed(event.id, &e.to_string())
                        .await?;
                }
            }
        }

        Ok(count)
    }
}
