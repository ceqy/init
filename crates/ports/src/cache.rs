//! Cache trait 定义

use async_trait::async_trait;
use cuba_errors::AppResult;
use std::time::Duration;

/// 缓存 trait
#[async_trait]
pub trait CachePort: Send + Sync {
    /// 获取缓存值
    async fn get(&self, key: &str) -> AppResult<Option<String>>;

    /// 设置缓存值
    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> AppResult<()>;

    /// 删除缓存
    async fn delete(&self, key: &str) -> AppResult<()>;

    /// 检查是否存在
    async fn exists(&self, key: &str) -> AppResult<bool>;

    /// 设置过期时间
    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()>;

    /// 原子性递增计数器，如果键不存在则创建并设置 TTL
    async fn incr_with_ttl(&self, key: &str, ttl_secs: u64) -> AppResult<i64> {
        // 默认实现（非原子性，子类应该覆盖）
        let current = self
            .get(key)
            .await?
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);
        let new_value = current + 1;
        self.set(
            key,
            &new_value.to_string(),
            Some(Duration::from_secs(ttl_secs)),
        )
        .await?;
        Ok(new_value)
    }

    /// 原子性递减计数器
    async fn decr(&self, key: &str) -> AppResult<i64> {
        // 默认实现（非原子性，子类应该覆盖）
        let current = self
            .get(key)
            .await?
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);
        let new_value = current - 1;
        self.set(key, &new_value.to_string(), None).await?;
        Ok(new_value)
    }

    /// 获取整数值
    async fn get_int(&self, key: &str) -> AppResult<Option<i64>> {
        Ok(self.get(key).await?.and_then(|v| v.parse().ok()))
    }

    /// 获取 TTL（秒）
    async fn ttl(&self, key: &str) -> AppResult<Option<i64>> {
        // 默认实现返回 None，子类应该覆盖
        let _ = key;
        Ok(None)
    }

    /// 使用 SETNX 实现分布式锁
    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> AppResult<bool> {
        // 默认实现（非原子性，子类应该覆盖）
        if self.exists(key).await? {
            Ok(false)
        } else {
            self.set(key, value, Some(ttl)).await?;
            Ok(true)
        }
    }

    /// 原子性地比较并删除
    async fn delete_if_equals(&self, key: &str, expected_value: &str) -> AppResult<bool> {
        // 默认实现（非原子性，子类应该覆盖）
        if let Some(current) = self.get(key).await?
            && current == expected_value
        {
            self.delete(key).await?;
            return Ok(true);
        }
        Ok(false)
    }
}

/// 分布式锁 trait
#[async_trait]
pub trait DistributedLock: Send + Sync {
    /// 获取锁
    async fn acquire(&self, key: &str, ttl: Duration) -> AppResult<bool>;

    /// 释放锁
    async fn release(&self, key: &str) -> AppResult<()>;
}
