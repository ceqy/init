//! Redis 分布式锁实现

use async_trait::async_trait;
use cuba_errors::{AppError, AppResult};
use cuba_ports::DistributedLock;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::time::Duration;

/// Redis 分布式锁
pub struct RedisDistributedLock {
    conn: ConnectionManager,
    lock_prefix: String,
}

impl RedisDistributedLock {
    pub fn new(conn: ConnectionManager) -> Self {
        Self {
            conn,
            lock_prefix: "lock:".to_string(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.lock_prefix = prefix.into();
        self
    }

    fn lock_key(&self, key: &str) -> String {
        format!("{}{}", self.lock_prefix, key)
    }
}

#[async_trait]
impl DistributedLock for RedisDistributedLock {
    async fn acquire(&self, key: &str, ttl: Duration) -> AppResult<bool> {
        let mut conn = self.conn.clone();
        let lock_key = self.lock_key(key);

        let result: Option<String> = redis::cmd("SET")
            .arg(&lock_key)
            .arg("1")
            .arg("NX")
            .arg("PX")
            .arg(ttl.as_millis() as u64)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Redis lock acquire failed: {}", e)))?;

        Ok(result.is_some())
    }

    async fn release(&self, key: &str) -> AppResult<()> {
        let mut conn = self.conn.clone();
        let lock_key = self.lock_key(key);

        conn.del(&lock_key)
            .await
            .map_err(|e| AppError::internal(format!("Redis lock release failed: {}", e)))
    }

    async fn with_lock<F, T>(&self, key: &str, ttl: Duration, f: F) -> AppResult<T>
    where
        F: FnOnce() -> AppResult<T> + Send,
        T: Send,
    {
        if !self.acquire(key, ttl).await? {
            return Err(AppError::conflict("Failed to acquire lock"));
        }

        let result = f();

        self.release(key).await?;

        result
    }
}
