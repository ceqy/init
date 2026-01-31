//! Kafka Producer
//!
//! 提供消息发布功能

use std::time::Duration;

use async_trait::async_trait;
use errors::{AppError, AppResult};
use ports::EventPublisher;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;
use serde::Serialize;
use tracing::debug;

use crate::config::ProducerConfig;

/// Kafka Producer 配置（简化版，保持向后兼容）
#[derive(Debug, Clone)]
pub struct KafkaProducerConfig {
    pub brokers: String,
    pub client_id: Option<String>,
}

impl KafkaProducerConfig {
    pub fn new(brokers: impl Into<String>) -> Self {
        Self {
            brokers: brokers.into(),
            client_id: None,
        }
    }

    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// 转换为完整配置
    pub fn to_producer_config(&self) -> ProducerConfig {
        let mut config = ProducerConfig::new(&self.brokers);
        if let Some(client_id) = &self.client_id {
            config = config.with_client_id(client_id);
        }
        config
    }
}

/// Kafka Event Publisher
pub struct KafkaEventPublisher {
    producer: FutureProducer,
    timeout: Duration,
}

impl KafkaEventPublisher {
    /// 从简化配置创建
    pub fn new(config: &KafkaProducerConfig) -> AppResult<Self> {
        Self::from_producer_config(&config.to_producer_config())
    }

    /// 从完整配置创建
    pub fn from_producer_config(config: &ProducerConfig) -> AppResult<Self> {
        let mut client_config = ClientConfig::new();

        for (key, value) in config.to_client_config_entries() {
            client_config.set(&key, &value);
        }

        let producer: FutureProducer = client_config
            .create()
            .map_err(|e| AppError::internal(format!("Failed to create Kafka producer: {}", e)))?;

        Ok(Self {
            producer,
            timeout: config.request_timeout,
        })
    }

    /// 从 broker 地址创建
    pub fn from_brokers(brokers: &str) -> AppResult<Self> {
        let config = ProducerConfig::new(brokers);
        Self::from_producer_config(&config)
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 发布带 key 的消息
    pub async fn publish_with_key<E: Serialize + Send + Sync>(
        &self,
        topic: &str,
        key: &str,
        event: &E,
    ) -> AppResult<(i32, i64)> {
        let payload = serde_json::to_string(event)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;

        let record = FutureRecord::to(topic).payload(&payload).key(key);

        let result = self
            .producer
            .send(record, Timeout::After(self.timeout))
            .await
            .map_err(|(e, _)| AppError::internal(format!("Failed to publish event: {}", e)))?;

        debug!(
            topic = topic,
            key = key,
            partition = result.0,
            offset = result.1,
            "Message published with key"
        );

        Ok(result)
    }

    /// 发布到指定分区
    pub async fn publish_to_partition<E: Serialize + Send + Sync>(
        &self,
        topic: &str,
        partition: i32,
        event: &E,
    ) -> AppResult<(i32, i64)> {
        let payload = serde_json::to_string(event)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;

        let record: FutureRecord<'_, str, String> = FutureRecord::to(topic)
            .payload(&payload)
            .partition(partition);

        let result = self
            .producer
            .send(record, Timeout::After(self.timeout))
            .await
            .map_err(|(e, _)| AppError::internal(format!("Failed to publish event: {}", e)))?;

        Ok(result)
    }

    /// 刷新所有待发送的消息
    pub fn flush(&self, timeout: Duration) {
        let _ = self.producer.flush(Timeout::After(timeout));
    }

    /// 获取内部 producer（用于高级操作）
    pub fn inner(&self) -> &FutureProducer {
        &self.producer
    }
}

#[async_trait]
impl EventPublisher for KafkaEventPublisher {
    async fn publish<E: Serialize + Send + Sync>(&self, topic: &str, event: &E) -> AppResult<()> {
        let payload = serde_json::to_string(event)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;
        self.publish_raw(topic, &payload).await
    }

    async fn publish_raw(&self, topic: &str, payload: &str) -> AppResult<()> {
        let record: FutureRecord<'_, str, str> = FutureRecord::to(topic).payload(payload);

        self.producer
            .send(record, Timeout::After(self.timeout))
            .await
            .map_err(|(e, _)| AppError::internal(format!("Failed to publish event: {}", e)))?;

        Ok(())
    }

    async fn publish_batch<E: Serialize + Send + Sync>(
        &self,
        topic: &str,
        events: &[E],
    ) -> AppResult<()> {
        use futures_util::future::join_all;

        let futures: Vec<_> = events
            .iter()
            .map(|event| async {
                let payload = serde_json::to_string(event)
                    .map_err(|e| AppError::internal(format!("Failed to serialize: {}", e)))?;

                let record: FutureRecord<'_, str, str> = FutureRecord::to(topic).payload(&payload);

                self.producer
                    .send(record, Timeout::After(self.timeout))
                    .await
                    .map_err(|(e, _)| AppError::internal(format!("Failed to publish: {}", e)))?;

                Ok::<(), AppError>(())
            })
            .collect();

        let results = join_all(futures).await;

        for result in results {
            result?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_producer_config() {
        let config = KafkaProducerConfig::new("localhost:9092").with_client_id("test-client");

        assert_eq!(config.brokers, "localhost:9092");
        assert_eq!(config.client_id, Some("test-client".to_string()));
    }

    #[tokio::test]
    #[ignore] // 需要 Kafka 实例
    async fn test_publisher() {
        let config = KafkaProducerConfig::new("localhost:9092");
        let publisher = KafkaEventPublisher::new(&config).unwrap();

        #[derive(Serialize)]
        struct TestEvent {
            message: String,
        }

        let event = TestEvent {
            message: "hello".to_string(),
        };

        publisher.publish("test-topic", &event).await.unwrap();
    }
}
