//! Kafka Consumer

use cuba_errors::{AppError, AppResult};
use futures_util::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};

/// Kafka Consumer 配置
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
}

/// DLQ 消息元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DlqMetadata {
    /// 原始 topic
    original_topic: String,
    /// 原始 partition
    original_partition: i32,
    /// 原始 offset
    original_offset: i64,
    /// 失败原因
    error_message: String,
    /// 重试次数
    retry_count: u32,
    /// 失败时间戳
    failed_at: i64,
}

/// DLQ 消息包装
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DlqMessage {
    /// 元数据
    metadata: DlqMetadata,
    /// 原始消息内容
    payload: String,
}

/// Kafka Event Consumer
pub struct KafkaEventConsumer {
    consumer: StreamConsumer,
    dlq_producer: Option<FutureProducer>,
    config: KafkaConsumerConfig,
}

impl KafkaEventConsumer {
    pub fn new(config: KafkaConsumerConfig) -> AppResult<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("group.id", &config.group_id)
            .set("enable.auto.commit", "false") // 禁用自动提交，改为手动提交
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(|e| AppError::internal(format!("Failed to create Kafka consumer: {}", e)))?;

        let topics: Vec<&str> = config.topics.iter().map(|s| s.as_str()).collect();
        consumer
            .subscribe(&topics)
            .map_err(|e| AppError::internal(format!("Failed to subscribe to topics: {}", e)))?;

        // 如果启用 DLQ，创建 producer
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

        Ok(Self {
            consumer,
            dlq_producer,
            config,
        })
    }

    /// 开始消费消息
    pub async fn start<F, Fut>(&self, handler: F) -> AppResult<()>
    where
        F: Fn(String, String) -> Fut + Send + Sync,
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
                            // 反序列化失败，直接发送到 DLQ
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
                            // 提交 offset，避免重复处理
                            if let Err(e) =
                                self.consumer.commit_message(&message, CommitMode::Async)
                            {
                                error!(
                                    "Failed to commit offset after deserialization error: {}",
                                    e
                                );
                            }
                            continue;
                        }
                        None => {
                            continue;
                        }
                    };

                    // 处理消息，带重试机制
                    if let Err(e) = self
                        .process_with_retry(
                            &handler,
                            topic.clone(),
                            payload.clone(),
                            partition,
                            offset,
                        )
                        .await
                    {
                        error!(
                            "Failed to process message from {} after {} retries: {}",
                            topic, self.config.max_retries, e
                        );

                        // 发送到 DLQ
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
                            // DLQ 发送失败，不提交 offset，下次重新处理
                            continue;
                        }
                    }

                    // 处理成功或已发送到 DLQ，提交 offset
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
        topic: String,
        payload: String,
        partition: i32,
        offset: i64,
    ) -> AppResult<()>
    where
        F: Fn(String, String) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = AppResult<()>> + Send,
    {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match handler(topic.clone(), payload.clone()).await {
                Ok(_) => {
                    if attempt > 0 {
                        info!(
                            "Message processed successfully after {} retries (topic: {}, partition: {}, offset: {})",
                            attempt, topic, partition, offset
                        );
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries {
                        warn!(
                            "Failed to process message (attempt {}/{}): {} (topic: {}, partition: {}, offset: {})",
                            attempt + 1,
                            self.config.max_retries + 1,
                            last_error.as_ref().unwrap(),
                            topic,
                            partition,
                            offset
                        );
                        // 指数退避：100ms, 200ms, 400ms...
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
            .key(original_topic); // 使用原始 topic 作为 key，便于分区

        dlq_producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| AppError::internal(format!("Failed to send to DLQ: {}", e)))?;

        warn!(
            "Message sent to DLQ: {} (original topic: {}, partition: {}, offset: {}, error: {})",
            dlq_topic, original_topic, partition, offset, error_message
        );

        Ok(())
    }
}
