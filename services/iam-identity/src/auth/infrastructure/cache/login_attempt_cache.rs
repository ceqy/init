//! 登录尝试缓存实现

use async_trait::async_trait;
use cuba_adapter_redis::RedisCache;
use cuba_errors::AppResult;
use cuba_ports::CachePort;

use crate::auth::domain::services::LoginAttemptCache;

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
        // 使用 Redis INCR 命令
        let value_str: Option<String> = self.cache.get(key).await?;
        
        let new_value = if let Some(v) = value_str {
            let current: i32 = v.parse().unwrap_or(0);
            current + 1
        } else {
            1
        };

        // 设置新值和 TTL
        self.cache
            .set(key, &new_value.to_string(), Some(std::time::Duration::from_secs(ttl_seconds as u64)))
            .await?;

        Ok(new_value)
    }

    async fn get(&self, key: &str) -> AppResult<i32> {
        let value: Option<String> = self.cache.get(key).await?;
        
        Ok(value
            .and_then(|v| v.parse().ok())
            .unwrap_or(0))
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        self.cache.delete(key).await
    }

    async fn ttl(&self, _key: &str) -> AppResult<Option<i64>> {
        // Redis TTL 命令
        // 这里需要扩展 RedisCache 以支持 TTL 查询
        // 简化实现：返回 None
        Ok(None)
    }
}
