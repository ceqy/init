//! Kafka Consumer

use cuba_errors::{AppError, AppResult};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use futures_util::StreamExt;
use tracing::{error, info};

/// Kafka Consumer 配置
#[derive(Debug, Clone)]
pub struct KafkaConsumerConfig {
    pub brokers: String,
    pub group_id: String,
    pub topics: Vec<String>,
}

impl KafkaConsumerConfig {
    pub fn new(brokers: impl Into<String>, group_id: impl Into<String>) -> Self {
        Self {
            brokers: brokers.into(),
            group_id: group_id.into(),
            topics: Vec::new(),
        }
    }

    pub fn with_topics(mut self, topics: Vec<String>) -> Self {
        self.topics = topics;
        self
    }
}

/// Kafka Event Consumer
pub struct KafkaEventConsumer {
    consumer: StreamConsumer,
}

impl KafkaEventConsumer {
    pub fn new(config: &KafkaConsumerConfig) -> AppResult<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("group.id", &config.group_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(|e| AppError::internal(format!("Failed to create Kafka consumer: {}", e)))?;

        let topics: Vec<&str> = config.topics.iter().map(|s| s.as_str()).collect();
        consumer
            .subscribe(&topics)
            .map_err(|e| AppError::internal(format!("Failed to subscribe to topics: {}", e)))?;

        Ok(Self { consumer })
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
                    let payload = match message.payload_view::<str>() {
                        Some(Ok(s)) => s.to_string(),
                        Some(Err(e)) => {
                            error!("Failed to deserialize message payload: {}", e);
                            continue;
                        }
                        None => {
                            continue;
                        }
                    };

                    if let Err(e) = handler(topic.clone(), payload).await {
                        error!("Failed to handle message from {}: {}", topic, e);
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
}
