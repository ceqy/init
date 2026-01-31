//! Kafka Consumer
//!
//! 提供消息消费功能，支持重试和 DLQ

use std::time::Duration;

use cuba_errors::{AppError, AppResult};
use futures_util::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::config::ConsumerConfig;

/// Kafka Consumer 配置（简化版，保持向后兼容）
#[derive(Debug, Clone)]
pub struct KafkaConsumerConfig {
    pub brokers: String,
    pub group_id: String,
    pub topics: Vec<String>,
    /// 最大重试次数（默认 3 次）
    pub max_retries: u32,
    /// 是否启用 DLQ（默认启用）
    pub enable_dlq: bool,
    /// DLQ topic 后缀（默认 ".dlq"）
    pub dlq_suffix: String,
}

impl KafkaConsumerConfig {
    pub fn new(brokers: impl Into<String>, group_id: impl Into<String>) -> Self {
        Self {
            brokers: brokers.into(),
            group_id: group_id.into(),
            topics: Vec::new(),
            max_retries: 3,
            enable_dlq: true,
            dlq_suffix: ".dlq".to_string(),
        }
    }

    pub fn with_topics(mut self, topics: Vec<String>) -> Self {
        self.topics = topics;
        self
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topics.push(topic.into());
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_dlq_enabled(mut self, enable_dlq: bool) -> Self {
        self.enable_dlq = enable_dlq;
        self
    }

    pub fn with_dlq_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.dlq_suffix = suffix.into();
        self
    }

    /// 转换为完整配置
    pub fn to_consumer_config(&self) -> ConsumerConfig {
        ConsumerConfig::new(&self.brokers, &self.group_id)
            .with_topics(self.topics.clone())
            .with_max_retries(self.max_retries)
            .with_dlq(self.enable_dlq)
            .with_dlq_suffix(&self.dlq_suffix)
    }
}

/// 消费的消息
#[derive(Debug, Clone)]
pub struct ConsumedMessage {
    /// Topic
    pub topic: String,
    /// 分区
    pub partition: i32,
    /// 偏移量
    pub offset: i64,
    /// 消息键
    pub key: Option<String>,
    /// 消息内容
    pub payload: String,
    /// 时间戳
    pub timestamp: Option<i64>,
}

impl ConsumedMessage {
    /// 解析 JSON 负载
    pub fn parse_payload<T: for<'de> Deserialize<'de>>(&self) -> AppResult<T> {
        serde_json::from_str(&self.payload)
            .map_err(|e| AppError::validation(format!("Failed to parse payload: {}", e)))
    }
}

/// DLQ 消息元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqMetadata {
    /// 原始 topic
    pub original_topic: String,
    /// 原始 partition
    pub original_partition: i32,
    /// 原始 offset
    pub original_offset: i64,
    /// 失败原因
    pub error_message: String,
    /// 重试次数
    pub retry_count: u32,
    /// 失败时间戳
    pub failed_at: i64,
}

/// DLQ 消息包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqMessage {
    /// 元数据
    pub metadata: DlqMetadata,
    /// 原始消息内容
    pub payload: String,
}

/// Kafka Event Consumer
pub struct KafkaEventConsumer {
    consumer: StreamConsumer,
    dlq_producer: Option<FutureProducer>,
    config: KafkaConsumerConfig,
}

impl KafkaEventConsumer {
    /// 从简化配置创建
    pub fn new(config: KafkaConsumerConfig) -> AppResult<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("group.id", &config.group_id)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(|e| AppError::internal(format!("Failed to create Kafka consumer: {}", e)))?;

        let topics: Vec<&str> = config.topics.iter().map(|s| s.as_str()).collect();
        consumer
            .subscribe(&topics)
            .map_err(|e| AppError::internal(format!("Failed to subscribe to topics: {}", e)))?;

        let dlq_producer = if config.enable_dlq {
            let producer: FutureProducer = ClientConfig::new()
                .set("bootstrap.servers", &config.brokers)
                .set("client.id", format!("{}-dlq-producer", config.group_id))
                .create()
                .map_err(|e| AppError::internal(format!("Failed to create DLQ producer: {}", e)))?;
            Some(producer)
        } else {
            None
        };

        info!(
            group_id = %config.group_id,
            topics = ?config.topics,
            "Kafka consumer created"
        );

        Ok(Self {
            consumer,
            dlq_producer,
            config,
        })
    }

    /// 从完整配置创建
    pub fn from_consumer_config(config: &ConsumerConfig) -> AppResult<Self> {
        let mut client_config = ClientConfig::new();

        for (key, value) in config.to_client_config_entries() {
            client_config.set(&key, &value);
        }

        let consumer: StreamConsumer = client_config
            .create()
            .map_err(|e| AppError::internal(format!("Failed to create Kafka consumer: {}", e)))?;

        let topics: Vec<&str> = config.topics.iter().map(|s| s.as_str()).collect();
        consumer
            .subscribe(&topics)
            .map_err(|e| AppError::internal(format!("Failed to subscribe to topics: {}", e)))?;

        let dlq_producer = if config.enable_dlq {
            let producer: FutureProducer = ClientConfig::new()
                .set("bootstrap.servers", &config.base.brokers)
                .set("client.id", format!("{}-dlq-producer", config.group_id))
                .create()
                .map_err(|e| AppError::internal(format!("Failed to create DLQ producer: {}", e)))?;
            Some(producer)
        } else {
            None
        };

        let simple_config = KafkaConsumerConfig {
            brokers: config.base.brokers.clone(),
            group_id: config.group_id.clone(),
            topics: config.topics.clone(),
            max_retries: config.max_retries,
            enable_dlq: config.enable_dlq,
            dlq_suffix: config.dlq_suffix.clone(),
        };

        Ok(Self {
            consumer,
            dlq_producer,
            config: simple_config,
        })
    }

    /// 开始消费消息（简单回调）
    pub async fn start<F, Fut>(&self, handler: F) -> AppResult<()>
    where
        F: Fn(String, String) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = AppResult<()>> + Send,
    {
        self.start_with_message(|msg| handler(msg.topic.clone(), msg.payload.clone()))
            .await
    }

    /// 开始消费消息（完整消息回调）
    pub async fn start_with_message<F, Fut>(&self, handler: F) -> AppResult<()>
    where
        F: Fn(ConsumedMessage) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = AppResult<()>> + Send,
    {
        let mut stream = self.consumer.stream();

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    let topic = message.topic().to_string();
                    let partition = message.partition();
                    let offset = message.offset();

                    let payload = match message.payload_view::<str>() {
                        Some(Ok(s)) => s.to_string(),
                        Some(Err(e)) => {
                            error!("Failed to deserialize message payload: {}", e);
                            if let Err(dlq_err) = self
                                .send_to_dlq(
                                    &topic,
                                    partition,
                                    offset,
                                    "",
                                    &format!("Deserialization error: {}", e),
                                    0,
                                )
                                .await
                            {
                                error!("Failed to send to DLQ: {}", dlq_err);
                            }
                            if let Err(e) =
                                self.consumer.commit_message(&message, CommitMode::Async)
                            {
                                error!("Failed to commit offset: {}", e);
                            }
                            continue;
                        }
                        None => {
                            debug!(topic = %topic, partition, offset, "Empty message, skipping");
                            continue;
                        }
                    };

                    let key = message
                        .key_view::<str>()
                        .and_then(|r| r.ok())
                        .map(|s| s.to_string());

                    let timestamp = message.timestamp().to_millis();

                    let consumed_msg = ConsumedMessage {
                        topic: topic.clone(),
                        partition,
                        offset,
                        key,
                        payload: payload.clone(),
                        timestamp,
                    };

                    if let Err(e) = self
                        .process_with_retry(&handler, consumed_msg, partition, offset)
                        .await
                    {
                        error!(
                            topic = %topic,
                            partition,
                            offset,
                            retries = self.config.max_retries,
                            error = %e,
                            "Failed to process message after retries"
                        );

                        if let Err(dlq_err) = self
                            .send_to_dlq(
                                &topic,
                                partition,
                                offset,
                                &payload,
                                &e.to_string(),
                                self.config.max_retries,
                            )
                            .await
                        {
                            error!("Failed to send to DLQ: {}", dlq_err);
                            continue;
                        }
                    }

                    if let Err(e) = self.consumer.commit_message(&message, CommitMode::Async) {
                        error!("Failed to commit offset: {}", e);
                    }
                }
                Err(e) => {
                    error!("Kafka error: {}", e);
                }
            }
        }

        info!("Kafka consumer stopped");
        Ok(())
    }

    /// 带重试机制的消息处理
    async fn process_with_retry<F, Fut>(
        &self,
        handler: &F,
        message: ConsumedMessage,
        partition: i32,
        offset: i64,
    ) -> AppResult<()>
    where
        F: Fn(ConsumedMessage) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = AppResult<()>> + Send,
    {
        let mut last_error = None;
        let topic = message.topic.clone();

        for attempt in 0..=self.config.max_retries {
            match handler(message.clone()).await {
                Ok(_) => {
                    if attempt > 0 {
                        info!(
                            topic = %topic,
                            partition,
                            offset,
                            attempt,
                            "Message processed successfully after retry"
                        );
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries {
                        warn!(
                            topic = %topic,
                            partition,
                            offset,
                            attempt = attempt + 1,
                            max_attempts = self.config.max_retries + 1,
                            error = %last_error.as_ref().unwrap(),
                            "Failed to process message, retrying"
                        );
                        let backoff = Duration::from_millis(100 * 2_u64.pow(attempt));
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// 发送消息到 DLQ
    async fn send_to_dlq(
        &self,
        original_topic: &str,
        partition: i32,
        offset: i64,
        payload: &str,
        error_message: &str,
        retry_count: u32,
    ) -> AppResult<()> {
        if !self.config.enable_dlq {
            warn!("DLQ is disabled, message will be lost");
            return Ok(());
        }

        let dlq_producer = self
            .dlq_producer
            .as_ref()
            .ok_or_else(|| AppError::internal("DLQ producer not initialized"))?;

        let dlq_topic = format!("{}{}", original_topic, self.config.dlq_suffix);

        let dlq_message = DlqMessage {
            metadata: DlqMetadata {
                original_topic: original_topic.to_string(),
                original_partition: partition,
                original_offset: offset,
                error_message: error_message.to_string(),
                retry_count,
                failed_at: chrono::Utc::now().timestamp(),
            },
            payload: payload.to_string(),
        };

        let dlq_payload = serde_json::to_string(&dlq_message)
            .map_err(|e| AppError::internal(format!("Failed to serialize DLQ message: {}", e)))?;

        let record: FutureRecord<'_, str, String> = FutureRecord::to(&dlq_topic)
            .payload(&dlq_payload)
            .key(original_topic);

        dlq_producer
            .send(record, Timeout::After(Duration::from_secs(5)))
            .await
            .map_err(|(e, _)| AppError::internal(format!("Failed to send to DLQ: {}", e)))?;

        warn!(
            dlq_topic = %dlq_topic,
            original_topic = %original_topic,
            partition,
            offset,
            error = %error_message,
            "Message sent to DLQ"
        );

        Ok(())
    }

    /// 获取消费者组 ID
    pub fn group_id(&self) -> &str {
        &self.config.group_id
    }

    /// 获取订阅的 topics
    pub fn topics(&self) -> &[String] {
        &self.config.topics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consumer_config() {
        let config = KafkaConsumerConfig::new("localhost:9092", "test-group")
            .with_topic("topic1")
            .with_topic("topic2")
            .with_max_retries(5);

        assert_eq!(config.topics.len(), 2);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_dlq_message() {
        let msg = DlqMessage {
            metadata: DlqMetadata {
                original_topic: "test".to_string(),
                original_partition: 0,
                original_offset: 100,
                error_message: "test error".to_string(),
                retry_count: 3,
                failed_at: 1234567890,
            },
            payload: "test payload".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: DlqMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.metadata.original_topic, "test");
    }

    #[tokio::test]
    #[ignore] // 需要 Kafka 实例
    async fn test_consumer() {
        let config = KafkaConsumerConfig::new("localhost:9092", "test-group")
            .with_topic("test-topic");

        let consumer = KafkaEventConsumer::new(config).unwrap();

        // 这里只是测试创建，实际消费需要 Kafka 实例
        assert_eq!(consumer.group_id(), "test-group");
    }
}
