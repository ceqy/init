//! Redis Cache 实现

use async_trait::async_trait;
use cuba_errors::{AppError, AppResult};
use cuba_ports::CachePort;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::time::Duration;

/// Redis Cache
pub struct RedisCache {
    conn: ConnectionManager,
}

impl RedisCache {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl CachePort for RedisCache {
    async fn get(&self, key: &str) -> AppResult<Option<String>> {
        let mut conn = self.conn.clone();
        conn.get(key)
            .await
            .map_err(|e| AppError::internal(format!("Redis get failed: {}", e)))
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> AppResult<()> {
        let mut conn = self.conn.clone();
        match ttl {
            Some(duration) => {
                conn.set_ex(key, value, duration.as_secs())
                    .await
                    .map_err(|e| AppError::internal(format!("Redis set failed: {}", e)))
            }
            None => conn
                .set(key, value)
                .await
                .map_err(|e| AppError::internal(format!("Redis set failed: {}", e))),
        }
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self.conn.clone();
        conn.del(key)
            .await
            .map_err(|e| AppError::internal(format!("Redis delete failed: {}", e)))
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.conn.clone();
        conn.exists(key)
            .await
            .map_err(|e| AppError::internal(format!("Redis exists failed: {}", e)))
    }

    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()> {
        let mut conn = self.conn.clone();
        conn.expire(key, ttl.as_secs() as i64)
            .await
            .map_err(|e| AppError::internal(format!("Redis expire failed: {}", e)))
    }
}
