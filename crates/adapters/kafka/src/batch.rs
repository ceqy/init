//! Kafka 批量生产模块
//!
//! 提供高效的批量消息发送功能

use std::time::Duration;

use cuba_errors::{AppError, AppResult};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;
use serde::Serialize;
use tracing::{debug, error, info};

use crate::config::ProducerConfig;

/// 批量消息
#[derive(Debug, Clone)]
pub struct BatchMessage {
    /// Topic
    pub topic: String,
    /// 消息键（可选）
    pub key: Option<String>,
    /// 消息内容
    pub payload: String,
    /// 分区（可选，不指定则由 Kafka 决定）
    pub partition: Option<i32>,
}

impl BatchMessage {
    pub fn new(topic: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            key: None,
            payload: payload.into(),
            partition: None,
        }
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_partition(mut self, partition: i32) -> Self {
        self.partition = Some(partition);
        self
    }

    /// 从可序列化对象创建
    pub fn from_json<T: Serialize>(topic: impl Into<String>, data: &T) -> AppResult<Self> {
        let payload = serde_json::to_string(data)
            .map_err(|e| AppError::internal(format!("Failed to serialize: {}", e)))?;
        Ok(Self::new(topic, payload))
    }
}

/// 批量发送结果
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// 成功数量
    pub success_count: usize,
    /// 失败数量
    pub failure_count: usize,
    /// 失败的消息索引和错误
    pub failures: Vec<(usize, String)>,
}

impl BatchResult {
    pub fn is_all_success(&self) -> bool {
        self.failure_count == 0
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            1.0
        } else {
            self.success_count as f64 / total as f64
        }
    }
}

/// 批量生产者
pub struct BatchProducer {
    producer: FutureProducer,
    timeout: Duration,
    max_concurrent: usize,
}

impl BatchProducer {
    /// 创建批量生产者
    pub fn new(config: &ProducerConfig) -> AppResult<Self> {
        let mut client_config = ClientConfig::new();

        for (key, value) in config.to_client_config_entries() {
            client_config.set(&key, &value);
        }

        let producer: FutureProducer = client_config
            .create()
            .map_err(|e| AppError::internal(format!("Failed to create producer: {}", e)))?;

        Ok(Self {
            producer,
            timeout: Duration::from_secs(10),
            max_concurrent: 1000,
        })
    }

    /// 从 broker 地址创建
    pub fn from_brokers(brokers: &str) -> AppResult<Self> {
        let config = ProducerConfig::new(brokers);
        Self::new(&config)
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置最大并发数
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// 发送单条消息
    pub async fn send(&self, message: &BatchMessage) -> AppResult<(i32, i64)> {
        let mut record: FutureRecord<'_, String, String> =
            FutureRecord::to(&message.topic).payload(&message.payload);

        if let Some(key) = &message.key {
            record = record.key(key);
        }

        if let Some(partition) = message.partition {
            record = record.partition(partition);
        }

        let result = self
            .producer
            .send(record, Timeout::After(self.timeout))
            .await
            .map_err(|(e, _)| AppError::internal(format!("Failed to send message: {}", e)))?;

        Ok(result)
    }

    /// 批量发送消息（并行）
    pub async fn send_batch(&self, messages: &[BatchMessage]) -> BatchResult {
        use futures_util::future::join_all;

        let mut success_count = 0;
        let mut failures = Vec::new();

        // 分批处理，避免同时发送太多
        for chunk in messages.chunks(self.max_concurrent) {
            let futures: Vec<_> = chunk
                .iter()
                .enumerate()
                .map(|(idx, msg)| async move { (idx, self.send(msg).await) })
                .collect();

            let results = join_all(futures).await;

            for (idx, result) in results {
                match result {
                    Ok((partition, offset)) => {
                        debug!(
                            partition = partition,
                            offset = offset,
                            "Message sent successfully"
                        );
                        success_count += 1;
                    }
                    Err(e) => {
                        error!(index = idx, error = %e, "Failed to send message");
                        failures.push((idx, e.to_string()));
                    }
                }
            }
        }

        let failure_count = failures.len();

        info!(
            success = success_count,
            failures = failure_count,
            "Batch send completed"
        );

        BatchResult {
            success_count,
            failure_count,
            failures,
        }
    }

    /// 发送到同一个 topic 的批量消息
    pub async fn send_to_topic<T: Serialize>(
        &self,
        topic: &str,
        items: &[T],
    ) -> AppResult<BatchResult> {
        let messages: Result<Vec<_>, _> = items
            .iter()
            .map(|item| BatchMessage::from_json(topic, item))
            .collect();

        let messages = messages?;
        Ok(self.send_batch(&messages).await)
    }

    /// 发送带 key 的批量消息
    pub async fn send_keyed<T: Serialize, K: AsRef<str>>(
        &self,
        topic: &str,
        items: &[(K, T)],
    ) -> AppResult<BatchResult> {
        let messages: Result<Vec<_>, _> = items
            .iter()
            .map(|(key, item)| {
                BatchMessage::from_json(topic, item).map(|m| m.with_key(key.as_ref()))
            })
            .collect();

        let messages = messages?;
        Ok(self.send_batch(&messages).await)
    }

    /// 刷新所有待发送的消息
    pub fn flush(&self, timeout: Duration) {
        let _ = self.producer.flush(Timeout::After(timeout));
    }
}

/// 消息构建器，用于流式构建批量消息
pub struct MessageBatchBuilder {
    messages: Vec<BatchMessage>,
    default_topic: Option<String>,
}

impl MessageBatchBuilder {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            default_topic: None,
        }
    }

    pub fn with_default_topic(mut self, topic: impl Into<String>) -> Self {
        self.default_topic = Some(topic.into());
        self
    }

    pub fn add(mut self, message: BatchMessage) -> Self {
        self.messages.push(message);
        self
    }

    pub fn add_payload(mut self, topic: impl Into<String>, payload: impl Into<String>) -> Self {
        self.messages.push(BatchMessage::new(topic, payload));
        self
    }

    pub fn add_json<T: Serialize>(mut self, topic: impl Into<String>, data: &T) -> AppResult<Self> {
        self.messages.push(BatchMessage::from_json(topic, data)?);
        Ok(self)
    }

    pub fn add_to_default<T: Serialize>(mut self, data: &T) -> AppResult<Self> {
        let topic = self
            .default_topic
            .clone()
            .ok_or_else(|| AppError::validation("Default topic not set"))?;
        self.messages.push(BatchMessage::from_json(topic, data)?);
        Ok(self)
    }

    pub fn build(self) -> Vec<BatchMessage> {
        self.messages
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl Default for MessageBatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_message() {
        let msg = BatchMessage::new("test-topic", "hello")
            .with_key("key1")
            .with_partition(0);

        assert_eq!(msg.topic, "test-topic");
        assert_eq!(msg.key, Some("key1".to_string()));
        assert_eq!(msg.partition, Some(0));
    }

    #[test]
    fn test_batch_result() {
        let result = BatchResult {
            success_count: 8,
            failure_count: 2,
            failures: vec![(3, "error".to_string())],
        };

        assert!(!result.is_all_success());
        assert!((result.success_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_message_builder() {
        let messages = MessageBatchBuilder::new()
            .add_payload("topic1", "msg1")
            .add_payload("topic2", "msg2")
            .build();

        assert_eq!(messages.len(), 2);
    }

    #[tokio::test]
    #[ignore] // 需要 Kafka 实例
    async fn test_batch_producer() {
        let producer = BatchProducer::from_brokers("localhost:9092").unwrap();

        let messages = vec![
            BatchMessage::new("test-topic", "message 1"),
            BatchMessage::new("test-topic", "message 2"),
            BatchMessage::new("test-topic", "message 3"),
        ];

        let result = producer.send_batch(&messages).await;
        println!("Result: {:?}", result);
    }
}
