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
}

/// 分布式锁 trait
#[async_trait]
pub trait DistributedLock: Send + Sync {
    /// 获取锁
    async fn acquire(&self, key: &str, ttl: Duration) -> AppResult<bool>;

    /// 释放锁
    async fn release(&self, key: &str) -> AppResult<()>;

    /// 带锁执行
    async fn with_lock<F, T>(&self, key: &str, ttl: Duration, f: F) -> AppResult<T>
    where
        F: FnOnce() -> AppResult<T> + Send,
        T: Send;
}
