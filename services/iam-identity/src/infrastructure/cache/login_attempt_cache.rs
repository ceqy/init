//! 登录尝试缓存实现

use async_trait::async_trait;
use cuba_adapter_redis::RedisCache;
use cuba_errors::AppResult;
use cuba_ports::CachePort;

use crate::domain::services::auth::LoginAttemptCache;

/// Redis 登录尝试缓存实现
pub struct RedisLoginAttemptCache {
    cache: RedisCache,
}

impl RedisLoginAttemptCache {
    pub fn new(cache: RedisCache) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl LoginAttemptCache for RedisLoginAttemptCache {
    async fn increment(&self, key: &str, ttl_seconds: i64) -> AppResult<i32> {
        // 使用原子性的 INCR + EXPIRE 操作（通过 Lua 脚本）
        let new_value = self.cache.incr_with_ttl(key, ttl_seconds as u64).await?;
        Ok(new_value as i32)
    }

    async fn get(&self, key: &str) -> AppResult<i32> {
        let value = self.cache.get_int(key).await?;
        Ok(value.unwrap_or(0) as i32)
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        self.cache.delete(key).await
    }

    async fn ttl(&self, key: &str) -> AppResult<Option<i64>> {
        self.cache.ttl(key).await
    }
}
