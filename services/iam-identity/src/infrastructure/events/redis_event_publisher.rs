use async_trait::async_trait;
use redis::{Client, AsyncCommands};
use tracing::{error, warn, debug};
use crate::infrastructure::events::{EventPublisher, IamDomainEvent};
use std::time::Duration;

/// Redis 事件发布器
///
/// 特性：
/// - 支持重试机制
/// - 连接失败时自动降级
/// - 记录发布指标
pub struct RedisEventPublisher {
    client: Client,
    channel_name: String,
    max_retries: u32,
    retry_delay: Duration,
}

impl RedisEventPublisher {
    pub fn new(redis_url: &str, channel_name: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client,
            channel_name: channel_name.to_string(),
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
        })
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 设置重试延迟
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// 带重试的发布
    async fn publish_with_retry(&self, payload: String) -> Result<(), redis::RedisError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                tokio::time::sleep(self.retry_delay * attempt).await;
            }

            let mut conn = match self.client.get_multiplexed_async_connection().await {
                Ok(conn) => conn,
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            };

            match conn.publish::<_, _, ()>(&self.channel_name, &payload).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        warn!(
                            attempt = attempt + 1,
                            max_retries = self.max_retries,
                            "Redis publish failed, retrying..."
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Failed to publish event after retries",
            ))
        }))
    }
}

#[async_trait]
impl EventPublisher for RedisEventPublisher {
    async fn publish(&self, event: IamDomainEvent) {
        let event_type = event.event_type();

        let payload = match serde_json::to_string(&event) {
            Ok(p) => p,
            Err(e) => {
                error!(
                    event_type = event_type,
                    error = %e,
                    "Failed to serialize event for Redis"
                );
                return;
            }
        };

        match self.publish_with_retry(payload).await {
            Ok(_) => {
                debug!(
                    event_type = event_type,
                    channel = %self.channel_name,
                    "Event published to Redis"
                );
            }
            Err(e) => {
                error!(
                    event_type = event_type,
                    channel = %self.channel_name,
                    error = %e,
                    "Failed to publish event to Redis after {} retries",
                    self.max_retries
                );
            }
        }
    }

    async fn publish_all(&self, events: Vec<IamDomainEvent>) {
        for event in events {
            self.publish(event).await;
        }
    }
}
