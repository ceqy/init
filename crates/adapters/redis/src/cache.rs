//! Redis Cache 实现

use async_trait::async_trait;
use errors::{AppError, AppResult};
use ports::CachePort;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Script};
use std::time::Duration;

/// Redis Cache
pub struct RedisCache {
    conn: ConnectionManager,
}

impl RedisCache {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    /// 原子性递增计数器，如果键不存在则创建并设置 TTL
    /// 返回递增后的值
    pub async fn incr_with_ttl(&self, key: &str, ttl_secs: u64) -> AppResult<i64> {
        let mut conn = self.conn.clone();

        // 使用 Lua 脚本确保原子性
        // 如果键不存在，设置为 1 并设置 TTL
        // 如果键存在，递增并保持原有 TTL
        let script = Script::new(
            r"
            local current = redis.call('INCR', KEYS[1])
            if current == 1 then
                redis.call('EXPIRE', KEYS[1], ARGV[1])
            end
            return current
            ",
        );

        script
            .key(key)
            .arg(ttl_secs)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Redis incr_with_ttl failed: {}", e)))
    }

    /// 原子性递减计数器
    pub async fn decr(&self, key: &str) -> AppResult<i64> {
        let mut conn = self.conn.clone();
        conn.decr(key, 1)
            .await
            .map_err(|e| AppError::internal(format!("Redis decr failed: {}", e)))
    }

    /// 获取整数值
    pub async fn get_int(&self, key: &str) -> AppResult<Option<i64>> {
        let mut conn = self.conn.clone();
        conn.get(key)
            .await
            .map_err(|e| AppError::internal(format!("Redis get_int failed: {}", e)))
    }

    /// 获取 TTL（秒），返回 None 表示键不存在或没有过期时间
    pub async fn ttl(&self, key: &str) -> AppResult<Option<i64>> {
        let mut conn = self.conn.clone();
        let ttl: i64 = conn
            .ttl(key)
            .await
            .map_err(|e| AppError::internal(format!("Redis ttl failed: {}", e)))?;

        // -2 表示键不存在，-1 表示没有过期时间
        match ttl {
            -2 => Ok(None),
            -1 => Ok(None),
            t => Ok(Some(t)),
        }
    }

    /// 使用 SETNX（SET if Not eXists）实现分布式锁
    /// 返回 true 表示获取锁成功，false 表示锁已被占用
    pub async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> AppResult<bool> {
        let mut conn = self.conn.clone();

        // 使用 SET NX EX 命令，原子性地设置键和过期时间
        let result: Option<String> = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("NX")
            .arg("EX")
            .arg(ttl.as_secs())
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Redis set_nx failed: {}", e)))?;

        Ok(result.is_some())
    }

    /// 使用 Lua 脚本原子性地比较并删除（用于释放分布式锁）
    /// 只有当值匹配时才删除，防止误删其他进程的锁
    pub async fn delete_if_equals(&self, key: &str, expected_value: &str) -> AppResult<bool> {
        let mut conn = self.conn.clone();

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
            .key(key)
            .arg(expected_value)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Redis delete_if_equals failed: {}", e)))?;

        Ok(deleted > 0)
    }

    /// 原子性地设置多个键值对（使用 MSET）
    pub async fn mset(&self, pairs: &[(&str, &str)]) -> AppResult<()> {
        let mut conn = self.conn.clone();

        let mut cmd = redis::cmd("MSET");
        for (key, value) in pairs {
            cmd.arg(key).arg(value);
        }

        cmd.query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Redis mset failed: {}", e)))
    }

    /// 原子性地获取多个键的值（使用 MGET）
    pub async fn mget(&self, keys: &[&str]) -> AppResult<Vec<Option<String>>> {
        let mut conn = self.conn.clone();
        conn.get(keys)
            .await
            .map_err(|e| AppError::internal(format!("Redis mget failed: {}", e)))
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
            Some(duration) => conn
                .set_ex(key, value, duration.as_secs())
                .await
                .map_err(|e| AppError::internal(format!("Redis set failed: {}", e))),
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

    async fn incr_with_ttl(&self, key: &str, ttl_secs: u64) -> AppResult<i64> {
        self.incr_with_ttl(key, ttl_secs).await
    }

    async fn decr(&self, key: &str) -> AppResult<i64> {
        self.decr(key).await
    }

    async fn get_int(&self, key: &str) -> AppResult<Option<i64>> {
        self.get_int(key).await
    }

    async fn ttl(&self, key: &str) -> AppResult<Option<i64>> {
        self.ttl(key).await
    }

    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> AppResult<bool> {
        self.set_nx(key, value, ttl).await
    }

    async fn delete_if_equals(&self, key: &str, expected_value: &str) -> AppResult<bool> {
        self.delete_if_equals(key, expected_value).await
    }
}
