//! Redis 分布式锁实现
//!
//! 使用 Redis SETNX 和 Lua 脚本实现分布式锁，防止误删其他进程的锁

use async_trait::async_trait;
use cuba_errors::{AppError, AppResult};
use cuba_ports::DistributedLock;
use redis::aio::ConnectionManager;
use redis::Script;
use std::time::Duration;
use uuid::Uuid;

/// Redis 分布式锁
pub struct RedisDistributedLock {
    conn: ConnectionManager,
    lock_prefix: String,
    /// 锁的唯一标识符（用于防止误删其他进程的锁）
    lock_id: String,
}

impl RedisDistributedLock {
    pub fn new(conn: ConnectionManager) -> Self {
        Self {
            conn,
            lock_prefix: "lock:".to_string(),
            lock_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.lock_prefix = prefix.into();
        self
    }

    fn lock_key(&self, key: &str) -> String {
        format!("{}{}", self.lock_prefix, key)
    }

    /// 尝试获取锁（带重试和指数退避）
    pub async fn acquire_with_retry(
        &self,
        key: &str,
        ttl: Duration,
        max_retries: u32,
        initial_delay_ms: u64,
    ) -> AppResult<bool> {
        let mut delay_ms = initial_delay_ms;

        for attempt in 0..=max_retries {
            if self.acquire(key, ttl).await? {
                return Ok(true);
            }

            if attempt < max_retries {
                // 指数退避，最大延迟 1 秒
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                delay_ms = (delay_ms * 2).min(1000);
            }
        }

        Ok(false)
    }
}

#[async_trait]
impl DistributedLock for RedisDistributedLock {
    async fn acquire(&self, key: &str, ttl: Duration) -> AppResult<bool> {
        let mut conn = self.conn.clone();
        let lock_key = self.lock_key(key);

        // 使用 SET NX EX 命令，原子性地设置键和过期时间
        // 存储唯一的 lock_id 而不是 "1"，用于后续安全释放
        let result: Option<String> = redis::cmd("SET")
            .arg(&lock_key)
            .arg(&self.lock_id)
            .arg("NX")
            .arg("EX")
            .arg(ttl.as_secs())
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Redis lock acquire failed: {}", e)))?;

        Ok(result.is_some())
    }

    async fn release(&self, key: &str) -> AppResult<()> {
        let mut conn = self.conn.clone();
        let lock_key = self.lock_key(key);

        // 使用 Lua 脚本确保只删除自己持有的锁
        // 这防止了误删其他进程的锁（例如当锁过期后被其他进程获取）
        let script = Script::new(
            r"
            if redis.call('GET', KEYS[1]) == ARGV[1] then
                return redis.call('DEL', KEYS[1])
            else
                return 0
            end
            ",
        );

        let deleted: i64 = script
            .key(&lock_key)
            .arg(&self.lock_id)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Redis lock release failed: {}", e)))?;

        if deleted == 0 {
            tracing::warn!(
                lock_key = %lock_key,
                lock_id = %self.lock_id,
                "Lock was not held or already expired"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要 Redis 实例
    async fn test_lock_acquire_and_release() {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let conn = ConnectionManager::new(client).await.unwrap();
        let lock = RedisDistributedLock::new(conn.clone());

        // 获取锁
        let acquired = lock
            .acquire("test_key", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(acquired);

        // 创建另一个锁实例（模拟另一个进程）
        let lock2 = RedisDistributedLock::new(conn);

        // 尝试再次获取同一个锁（应该失败）
        let acquired_again = lock2
            .acquire("test_key", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(!acquired_again);

        // 释放锁
        lock.release("test_key").await.unwrap();

        // 再次获取锁（应该成功）
        let acquired_after_release = lock2
            .acquire("test_key", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(acquired_after_release);

        // 清理
        lock2.release("test_key").await.unwrap();
    }

    #[tokio::test]
    #[ignore] // 需要 Redis 实例
    async fn test_lock_prevents_accidental_deletion() {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let conn = ConnectionManager::new(client).await.unwrap();
        let lock1 = RedisDistributedLock::new(conn.clone());
        let lock2 = RedisDistributedLock::new(conn);

        // lock1 获取锁
        assert!(lock1.acquire("test_key", Duration::from_secs(1)).await.unwrap());

        // 等待锁过期
        tokio::time::sleep(Duration::from_secs(2)).await;

        // lock2 获取锁（此时 lock1 的锁已过期）
        assert!(lock2.acquire("test_key", Duration::from_secs(10)).await.unwrap());

        // lock1 尝试释放锁（应该不会删除 lock2 的锁）
        lock1.release("test_key").await.unwrap();

        // lock2 应该仍然持有锁
        assert!(!lock1.acquire("test_key", Duration::from_secs(1)).await.unwrap());

        // 清理
        lock2.release("test_key").await.unwrap();
    }

    #[tokio::test]
    #[ignore] // 需要 Redis 实例
    async fn test_with_lock() {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let conn = ConnectionManager::new(client).await.unwrap();
        let lock = RedisDistributedLock::new(conn);

        // 获取锁
        assert!(lock.acquire("test_key", Duration::from_secs(10)).await.unwrap());

        // 执行业务逻辑
        let result = 42;

        // 释放锁
        lock.release("test_key").await.unwrap();

        assert_eq!(result, 42);
    }

    #[tokio::test]
    #[ignore] // 需要 Redis 实例
    async fn test_acquire_with_retry() {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let conn = ConnectionManager::new(client).await.unwrap();
        let lock1 = RedisDistributedLock::new(conn.clone());
        let lock2 = RedisDistributedLock::new(conn);

        // lock1 获取锁，短 TTL
        assert!(lock1.acquire("test_key", Duration::from_millis(500)).await.unwrap());

        // lock2 尝试获取锁，带重试
        let acquired = lock2
            .acquire_with_retry("test_key", Duration::from_secs(10), 5, 100)
            .await
            .unwrap();

        // 应该在重试后成功获取锁
        assert!(acquired);

        // 清理
        lock2.release("test_key").await.unwrap();
    }
}
