use async_trait::async_trait;
use cuba_errors::AppResult;
use cuba_ports::EventPublisher;
use redis::{AsyncCommands, Client};
use serde::Serialize;
use tracing::warn;

pub struct RedisEventPublisher {
    client: Client,
    _base_channel: String, // fallback if needed, or prefix?
}

impl RedisEventPublisher {
    pub fn new(redis_url: &str, base_channel: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client,
            _base_channel: base_channel.to_string(),
        })
    }
}

#[async_trait]
impl EventPublisher for RedisEventPublisher {
    async fn publish<E: Serialize + Send + Sync>(&self, topic: &str, event: &E) -> AppResult<()> {
        let payload = serde_json::to_string(event)
            .map_err(|e| cuba_errors::AppError::internal(e.to_string()))?;
        self.publish_raw(topic, &payload).await
    }

    async fn publish_raw(&self, topic: &str, payload: &str) -> AppResult<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                cuba_errors::AppError::internal(format!("Redis connection failed: {}", e))
            })?;

        conn.publish::<_, _, ()>(topic, payload)
            .await
            .map_err(|e| cuba_errors::AppError::internal(format!("Redis publish failed: {}", e)))?;

        Ok(())
    }

    async fn publish_batch<E: Serialize + Send + Sync>(
        &self,
        topic: &str,
        events: &[E],
    ) -> AppResult<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                cuba_errors::AppError::internal(format!("Redis connection failed: {}", e))
            })?;

        // Pipeline support would be better, but loop is simple for now
        for event in events {
            let payload = serde_json::to_string(event)
                .map_err(|e| cuba_errors::AppError::internal(e.to_string()))?;
            if let Err(e) = conn.publish::<_, _, ()>(topic, payload).await {
                warn!("Failed to publish event in batch: {}", e);
                // Continue or fail? Trait says AppResult needed.
                // Let's fail fast for now.
                return Err(cuba_errors::AppError::internal(format!(
                    "Redis batch publish failed: {}",
                    e
                )));
            }
        }
        Ok(())
    }
}
