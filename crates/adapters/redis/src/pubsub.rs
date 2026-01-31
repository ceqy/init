//! Redis 发布/订阅模块
//!
//! 提供 Pub/Sub 功能

use cuba_errors::{AppError, AppResult};
use futures::StreamExt;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, error, info};

use crate::config::RedisConfig;

/// 发布的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubMessage {
    /// 频道
    pub channel: String,
    /// 消息内容
    pub payload: String,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PubSubMessage {
    /// 创建新消息
    pub fn new(channel: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
            payload: payload.into(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// 解析 JSON 负载
    pub fn parse_payload<T: for<'de> Deserialize<'de>>(&self) -> AppResult<T> {
        serde_json::from_str(&self.payload)
            .map_err(|e| AppError::validation(format!("Failed to parse message payload: {}", e)))
    }
}

/// Redis 发布者
pub struct RedisPublisher {
    conn: ConnectionManager,
    key_prefix: Option<String>,
}

impl RedisPublisher {
    /// 创建新的发布者
    pub async fn new(config: &RedisConfig) -> AppResult<Self> {
        let client = Client::open(config.url.as_str())
            .map_err(|e| AppError::internal(format!("Failed to create Redis client: {}", e)))?;

        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::internal(format!("Failed to create connection: {}", e)))?;

        Ok(Self {
            conn,
            key_prefix: config.key_prefix.clone(),
        })
    }

    /// 从连接管理器创建
    pub fn from_connection(conn: ConnectionManager) -> Self {
        Self {
            conn,
            key_prefix: None,
        }
    }

    /// 设置键前缀
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// 获取带前缀的频道名
    fn prefixed_channel(&self, channel: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, channel),
            None => channel.to_string(),
        }
    }

    /// 发布消息
    pub async fn publish(&mut self, channel: &str, message: &str) -> AppResult<u64> {
        let channel = self.prefixed_channel(channel);
        let subscribers: u64 = self
            .conn
            .publish(&channel, message)
            .await
            .map_err(|e| AppError::internal(format!("Failed to publish message: {}", e)))?;

        debug!(channel = %channel, subscribers, "Message published");
        Ok(subscribers)
    }

    /// 发布 JSON 消息
    pub async fn publish_json<T: Serialize>(&mut self, channel: &str, data: &T) -> AppResult<u64> {
        let payload = serde_json::to_string(data)
            .map_err(|e| AppError::internal(format!("Failed to serialize message: {}", e)))?;
        self.publish(channel, &payload).await
    }
}

/// Redis 订阅者
pub struct RedisSubscriber {
    client: Client,
    key_prefix: Option<String>,
}

impl RedisSubscriber {
    /// 创建新的订阅者
    pub async fn new(config: &RedisConfig) -> AppResult<Self> {
        let client = Client::open(config.url.as_str())
            .map_err(|e| AppError::internal(format!("Failed to create Redis client: {}", e)))?;

        Ok(Self {
            client,
            key_prefix: config.key_prefix.clone(),
        })
    }

    /// 设置键前缀
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// 获取带前缀的频道名
    fn prefixed_channel(&self, channel: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, channel),
            None => channel.to_string(),
        }
    }

    /// 订阅频道并处理消息
    pub async fn subscribe<F, Fut>(
        &self,
        channels: &[&str],
        mut handler: F,
    ) -> AppResult<()>
    where
        F: FnMut(PubSubMessage) -> Fut + Send,
        Fut: std::future::Future<Output = AppResult<()>> + Send,
    {
        let prefixed_channels: Vec<String> = channels
            .iter()
            .map(|c| self.prefixed_channel(c))
            .collect();

        let mut pubsub = self
            .client
            .get_async_pubsub()
            .await
            .map_err(|e| AppError::internal(format!("Failed to get pubsub connection: {}", e)))?;

        for channel in &prefixed_channels {
            pubsub
                .subscribe(channel)
                .await
                .map_err(|e| AppError::internal(format!("Failed to subscribe to {}: {}", channel, e)))?;
        }

        info!(channels = ?prefixed_channels, "Subscribed to channels");

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let channel: String = msg.get_channel_name().to_string();
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    error!(error = %e, "Failed to get message payload");
                    continue;
                }
            };

            let message = PubSubMessage::new(channel.clone(), payload);

            if let Err(e) = handler(message).await {
                error!(channel = %channel, error = %e, "Failed to handle message");
            }
        }

        Ok(())
    }

    /// 订阅模式匹配的频道
    pub async fn psubscribe<F, Fut>(
        &self,
        patterns: &[&str],
        mut handler: F,
    ) -> AppResult<()>
    where
        F: FnMut(PubSubMessage) -> Fut + Send,
        Fut: std::future::Future<Output = AppResult<()>> + Send,
    {
        let prefixed_patterns: Vec<String> = patterns
            .iter()
            .map(|p| self.prefixed_channel(p))
            .collect();

        let mut pubsub = self
            .client
            .get_async_pubsub()
            .await
            .map_err(|e| AppError::internal(format!("Failed to get pubsub connection: {}", e)))?;

        for pattern in &prefixed_patterns {
            pubsub
                .psubscribe(pattern)
                .await
                .map_err(|e| AppError::internal(format!("Failed to psubscribe to {}: {}", pattern, e)))?;
        }

        info!(patterns = ?prefixed_patterns, "Subscribed to patterns");

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let channel: String = msg.get_channel_name().to_string();
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    error!(error = %e, "Failed to get message payload");
                    continue;
                }
            };

            let message = PubSubMessage::new(channel.clone(), payload);

            if let Err(e) = handler(message).await {
                error!(channel = %channel, error = %e, "Failed to handle message");
            }
        }

        Ok(())
    }
}

/// Pub/Sub 管理器（支持多订阅者）
pub struct PubSubManager {
    publisher: RedisPublisher,
    sender: broadcast::Sender<PubSubMessage>,
}

impl PubSubManager {
    /// 创建新的管理器
    pub async fn new(config: &RedisConfig, capacity: usize) -> AppResult<Self> {
        let publisher = RedisPublisher::new(config).await?;
        let (sender, _) = broadcast::channel(capacity);

        Ok(Self { publisher, sender })
    }

    /// 发布消息
    pub async fn publish(&mut self, channel: &str, message: &str) -> AppResult<u64> {
        self.publisher.publish(channel, message).await
    }

    /// 获取订阅接收器
    pub fn subscribe(&self) -> broadcast::Receiver<PubSubMessage> {
        self.sender.subscribe()
    }

    /// 广播本地消息（不经过 Redis）
    pub fn broadcast(&self, message: PubSubMessage) -> Result<usize, broadcast::error::SendError<PubSubMessage>> {
        self.sender.send(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pubsub_message() {
        let msg = PubSubMessage::new("test-channel", r#"{"key": "value"}"#);
        assert_eq!(msg.channel, "test-channel");

        #[derive(Deserialize)]
        struct TestData {
            key: String,
        }

        let data: TestData = msg.parse_payload().unwrap();
        assert_eq!(data.key, "value");
    }

    #[tokio::test]
    #[ignore] // 需要 Redis 实例
    async fn test_publisher() {
        let config = RedisConfig::new("redis://127.0.0.1:6379");
        let mut publisher = RedisPublisher::new(&config).await.unwrap();

        let subscribers = publisher.publish("test-channel", "hello").await.unwrap();
        // 没有订阅者时返回 0
        assert_eq!(subscribers, 0);
    }
}
