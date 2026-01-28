use std::sync::Arc;
use async_trait::async_trait;
use redis::{Client, AsyncCommands};
use tracing::error;
use crate::infrastructure::events::{EventPublisher, IamDomainEvent};

/// Redis 事件发布器
pub struct RedisEventPublisher {
    client: Client,
    channel_name: String,
}

impl RedisEventPublisher {
    pub fn new(redis_url: &str, channel_name: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client,
            channel_name: channel_name.to_string(),
        })
    }
}

#[async_trait]
impl EventPublisher for RedisEventPublisher {
    async fn publish(&self, event: IamDomainEvent) {
        let payload = match serde_json::to_string(&event) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to serialize event for Redis: {}", e);
                return;
            }
        };

        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return;
            }
        };

        if let Err(e) = conn.publish::<_, _, ()>(&self.channel_name, payload).await {
            error!("Failed to publish event to Redis channel {}: {}", self.channel_name, e);
        } else {
             tracing::debug!(
                event_type = event.event_type(),
                channel = %self.channel_name,
                "Event published to Redis"
            );
        }
    }

    async fn publish_all(&self, events: Vec<IamDomainEvent>) {
        for event in events {
            self.publish(event).await;
        }
    }
}
