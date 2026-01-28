//! Kafka Producer

use async_trait::async_trait;
use cuba_errors::{AppError, AppResult};
use cuba_ports::EventPublisher;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;
use std::time::Duration;

/// Kafka Producer 配置
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
}

/// Kafka Event Publisher
pub struct KafkaEventPublisher {
    producer: FutureProducer,
}

impl KafkaEventPublisher {
    pub fn new(config: &KafkaProducerConfig) -> AppResult<Self> {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", &config.brokers);

        if let Some(client_id) = &config.client_id {
            client_config.set("client.id", client_id);
        }

        let producer: FutureProducer = client_config
            .create()
            .map_err(|e| AppError::internal(format!("Failed to create Kafka producer: {}", e)))?;

        Ok(Self { producer })
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
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| AppError::internal(format!("Failed to publish event: {}", e)))?;

        Ok(())
    }

    async fn publish_batch<E: Serialize + Send + Sync>(
        &self,
        topic: &str,
        events: &[E],
    ) -> AppResult<()> {
        for event in events {
            self.publish(topic, event).await?;
        }
        Ok(())
    }
}
