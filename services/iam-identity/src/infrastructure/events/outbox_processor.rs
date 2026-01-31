//! Outbox 后台处理器
//!
//! 定期扫描 outbox 表并发布未处理的消息

use errors::AppResult;
use ports::OutboxPort;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info};

use super::event_publisher::{EventPublisher, IamDomainEvent};

/// Outbox 处理器配置
#[derive(Debug, Clone)]
pub struct OutboxProcessorConfig {
    /// 扫描间隔
    pub scan_interval: Duration,
    /// 每批处理的消息数量
    pub batch_size: usize,
    /// 清理已处理消息的保留时间
    pub retention_period: Duration,
}

impl Default for OutboxProcessorConfig {
    fn default() -> Self {
        Self {
            scan_interval: Duration::from_secs(5),
            batch_size: 100,
            retention_period: Duration::from_secs(86400 * 7), // 7 天
        }
    }
}

/// Outbox 处理器
///
/// 负责：
/// 1. 定期扫描 outbox 表
/// 2. 发布未处理的消息
/// 3. 标记已处理的消息
/// 4. 清理过期的已处理消息
pub struct OutboxProcessor {
    outbox: Arc<dyn OutboxPort>,
    publisher: Arc<dyn EventPublisher>,
    config: OutboxProcessorConfig,
}

impl OutboxProcessor {
    /// 创建新的 Outbox 处理器
    pub fn new(
        outbox: Arc<dyn OutboxPort>,
        publisher: Arc<dyn EventPublisher>,
        config: OutboxProcessorConfig,
    ) -> Self {
        Self {
            outbox,
            publisher,
            config,
        }
    }

    /// 启动后台处理任务
    pub fn start(
        self: Arc<Self>,
        shutdown: tokio_util::sync::CancellationToken,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Outbox processor started");
            let mut scan_ticker = interval(self.config.scan_interval);
            let mut cleanup_ticker = interval(Duration::from_secs(3600)); // 每小时清理一次

            loop {
                tokio::select! {
                    _ = scan_ticker.tick() => {
                        if let Err(e) = self.process_pending_messages().await {
                            error!(error = %e, "Failed to process pending messages");
                        }
                    }
                    _ = cleanup_ticker.tick() => {
                        if let Err(e) = self.cleanup_processed_messages().await {
                            error!(error = %e, "Failed to cleanup processed messages");
                        }
                    }
                    _ = shutdown.cancelled() => {
                        info!("Outbox processor received shutdown signal");
                        break;
                    }
                }
            }
            info!("Outbox processor stopped");
        })
    }

    /// 处理待处理的消息
    async fn process_pending_messages(&self) -> AppResult<()> {
        let messages = self.outbox.get_pending(self.config.batch_size).await?;

        if messages.is_empty() {
            return Ok(());
        }

        debug!(count = messages.len(), "Processing pending outbox messages");

        let mut processed_count = 0;
        let mut failed_count = 0;

        for message in messages {
            match self.process_message(&message).await {
                Ok(_) => {
                    // 标记为已处理
                    if let Err(e) = self.outbox.mark_processed(&message.id).await {
                        error!(
                            message_id = %message.id,
                            error = %e,
                            "Failed to mark message as processed"
                        );
                        failed_count += 1;
                    } else {
                        processed_count += 1;
                    }
                }
                Err(e) => {
                    error!(
                        message_id = %message.id,
                        event_type = %message.event_type,
                        error = %e,
                        "Failed to process outbox message"
                    );
                    failed_count += 1;
                }
            }
        }

        if processed_count > 0 {
            info!(
                processed = processed_count,
                failed = failed_count,
                "Outbox messages processed"
            );
        }

        Ok(())
    }

    /// 处理单个消息
    async fn process_message(&self, message: &ports::OutboxMessage) -> AppResult<()> {
        // 反序列化事件
        let event: IamDomainEvent = serde_json::from_str(&message.payload).map_err(|e| {
            errors::AppError::internal(format!("Failed to deserialize event: {}", e))
        })?;

        // 发布事件
        self.publisher.publish(event).await;

        Ok(())
    }

    /// 清理已处理的消息
    async fn cleanup_processed_messages(&self) -> AppResult<()> {
        let before = chrono::Utc::now()
            - chrono::Duration::from_std(self.config.retention_period).map_err(|e| {
                errors::AppError::internal(format!("Invalid retention period: {}", e))
            })?;

        let deleted = self.outbox.delete_processed(before).await?;

        if deleted > 0 {
            info!(count = deleted, "Cleaned up processed outbox messages");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use ports::OutboxMessage;
    use std::sync::Mutex;

    struct MockOutbox {
        messages: Arc<Mutex<Vec<OutboxMessage>>>,
        processed: Arc<Mutex<Vec<String>>>,
    }

    impl MockOutbox {
        fn new() -> Self {
            Self {
                messages: Arc::new(Mutex::new(Vec::new())),
                processed: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn add_message(&self, message: OutboxMessage) {
            self.messages.lock().unwrap().push(message);
        }
    }

    #[async_trait]
    impl OutboxPort for MockOutbox {
        async fn save(&self, message: &OutboxMessage) -> AppResult<()> {
            self.messages.lock().unwrap().push(message.clone());
            Ok(())
        }

        async fn get_pending(&self, limit: usize) -> AppResult<Vec<OutboxMessage>> {
            let messages = self.messages.lock().unwrap();
            let processed = self.processed.lock().unwrap();

            Ok(messages
                .iter()
                .filter(|m| !processed.contains(&m.id))
                .take(limit)
                .cloned()
                .collect())
        }

        async fn mark_processed(&self, id: &str) -> AppResult<()> {
            self.processed.lock().unwrap().push(id.to_string());
            Ok(())
        }

        async fn delete_processed(&self, _before: chrono::DateTime<chrono::Utc>) -> AppResult<u64> {
            Ok(0)
        }
    }

    struct MockPublisher {
        published: Arc<Mutex<Vec<String>>>,
    }

    impl MockPublisher {
        fn new() -> Self {
            Self {
                published: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl EventPublisher for MockPublisher {
        async fn publish(&self, event: IamDomainEvent) {
            self.published
                .lock()
                .unwrap()
                .push(event.event_type().to_string());
        }

        async fn publish_all(&self, events: Vec<IamDomainEvent>) {
            for event in events {
                self.publish(event).await;
            }
        }
    }

    #[tokio::test]
    async fn test_process_pending_messages() {
        use chrono::Utc;
        use common::{TenantId, UserId};

        let outbox = Arc::new(MockOutbox::new());
        let publisher = Arc::new(MockPublisher::new());

        // 添加测试消息
        let event = IamDomainEvent::UserCreated {
            user_id: UserId::new(),
            tenant_id: TenantId::new(),
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            timestamp: Utc::now(),
        };

        let message = OutboxMessage {
            id: "test-1".to_string(),
            aggregate_type: "User".to_string(),
            aggregate_id: "user-1".to_string(),
            event_type: "UserCreated".to_string(),
            payload: serde_json::to_string(&event).unwrap(),
            created_at: Utc::now(),
            processed_at: None,
        };

        outbox.add_message(message);

        let processor = OutboxProcessor::new(
            outbox.clone(),
            publisher.clone(),
            OutboxProcessorConfig::default(),
        );

        // 处理消息
        processor.process_pending_messages().await.unwrap();

        // 验证消息已发布
        assert_eq!(publisher.published.lock().unwrap().len(), 1);
        assert_eq!(publisher.published.lock().unwrap()[0], "UserCreated");

        // 验证消息已标记为已处理
        assert_eq!(outbox.processed.lock().unwrap().len(), 1);
        assert_eq!(outbox.processed.lock().unwrap()[0], "test-1");
    }
}
