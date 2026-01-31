//! 多层缓存实现
//!
//! 提供本地内存缓存 + Redis 缓存的两层架构
//! - L1: 本地内存缓存（快速，但不共享）
//! - L2: Redis 缓存（共享，但较慢）
//!
//! 当 Redis 故障时，自动降级到本地缓存

use async_trait::async_trait;
use errors::AppResult;
use ports::CachePort;
use moka::future::Cache as MokaCache;
use std::sync::Arc;
use std::time::Duration;

/// 多层缓存配置
#[derive(Clone)]
pub struct MultiLayerCacheConfig {
    /// L1 缓存最大条目数
    pub l1_max_capacity: u64,
    /// L1 缓存 TTL（秒）
    pub l1_ttl_secs: u64,
    /// 是否在 L2 失败时降级到 L1
    pub fallback_to_l1: bool,
}

impl Default for MultiLayerCacheConfig {
    fn default() -> Self {
        Self {
            l1_max_capacity: 10_000,
            l1_ttl_secs: 60, // L1 缓存 1 分钟
            fallback_to_l1: true,
        }
    }
}

/// 多层缓存
/// L1: 本地内存缓存（Moka）
/// L2: Redis 缓存
#[derive(Clone)]
pub struct MultiLayerCache {
    /// L1 缓存（本地内存）
    l1: MokaCache<String, String>,
    /// L2 缓存（Redis）
    l2: Arc<dyn CachePort>,
    /// 配置
    config: MultiLayerCacheConfig,
}

impl MultiLayerCache {
    pub fn new(l2: Arc<dyn CachePort>, config: MultiLayerCacheConfig) -> Self {
        let l1 = MokaCache::builder()
            .max_capacity(config.l1_max_capacity)
            .time_to_live(Duration::from_secs(config.l1_ttl_secs))
            .build();

        Self { l1, l2, config }
    }

    /// 从 L1 获取
    async fn get_from_l1(&self, key: &str) -> Option<String> {
        self.l1.get(key).await
    }

    /// 写入 L1
    async fn set_to_l1(&self, key: &str, value: &str) {
        self.l1.insert(key.to_string(), value.to_string()).await;
    }

    /// 从 L1 删除
    async fn delete_from_l1(&self, key: &str) {
        self.l1.invalidate(key).await;
    }
}

#[async_trait]
impl CachePort for MultiLayerCache {
    async fn get(&self, key: &str) -> AppResult<Option<String>> {
        // 1. 先查 L1
        if let Some(value) = self.get_from_l1(key).await {
            tracing::debug!(key = %key, "Cache hit in L1");
            return Ok(Some(value));
        }

        // 2. 查 L2
        match self.l2.get(key).await {
            Ok(Some(value)) => {
                tracing::debug!(key = %key, "Cache hit in L2");
                // 回填 L1
                self.set_to_l1(key, &value).await;
                Ok(Some(value))
            }
            Ok(None) => {
                tracing::debug!(key = %key, "Cache miss");
                Ok(None)
            }
            Err(e) => {
                tracing::warn!(key = %key, error = %e, "L2 cache error");
                if self.config.fallback_to_l1 {
                    // 降级到 L1
                    Ok(self.get_from_l1(key).await)
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> AppResult<()> {
        // 同时写入 L1 和 L2
        self.set_to_l1(key, value).await;

        match self.l2.set(key, value, ttl).await {
            Ok(()) => Ok(()),
            Err(e) => {
                tracing::warn!(key = %key, error = %e, "L2 cache set error");
                if self.config.fallback_to_l1 {
                    // 降级模式：只写 L1 也算成功
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        // 同时删除 L1 和 L2
        self.delete_from_l1(key).await;

        match self.l2.delete(key).await {
            Ok(()) => Ok(()),
            Err(e) => {
                tracing::warn!(key = %key, error = %e, "L2 cache delete error");
                if self.config.fallback_to_l1 {
                    // 降级模式：只删 L1 也算成功
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        // 先查 L1
        if self.l1.contains_key(key) {
            return Ok(true);
        }

        // 再查 L2
        match self.l2.exists(key).await {
            Ok(exists) => Ok(exists),
            Err(e) => {
                tracing::warn!(key = %key, error = %e, "L2 cache exists error");
                if self.config.fallback_to_l1 {
                    Ok(self.l1.contains_key(key))
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()> {
        // L1 不支持动态修改 TTL，只操作 L2
        self.l2.expire(key, ttl).await
    }

    async fn incr_with_ttl(&self, key: &str, ttl_secs: u64) -> AppResult<i64> {
        // 计数器操作只在 L2 进行（需要原子性）
        let result = self.l2.incr_with_ttl(key, ttl_secs).await?;
        // 使 L1 失效
        self.delete_from_l1(key).await;
        Ok(result)
    }

    async fn decr(&self, key: &str) -> AppResult<i64> {
        // 计数器操作只在 L2 进行（需要原子性）
        let result = self.l2.decr(key).await?;
        // 使 L1 失效
        self.delete_from_l1(key).await;
        Ok(result)
    }

    async fn get_int(&self, key: &str) -> AppResult<Option<i64>> {
        // 整数值不缓存在 L1（避免不一致）
        self.l2.get_int(key).await
    }

    async fn ttl(&self, key: &str) -> AppResult<Option<i64>> {
        // TTL 只在 L2 有意义
        self.l2.ttl(key).await
    }

    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> AppResult<bool> {
        // SETNX 只在 L2 进行（需要原子性）
        self.l2.set_nx(key, value, ttl).await
    }

    async fn delete_if_equals(&self, key: &str, expected_value: &str) -> AppResult<bool> {
        // 原子操作只在 L2 进行
        let result = self.l2.delete_if_equals(key, expected_value).await?;
        if result {
            // 删除成功，使 L1 失效
            self.delete_from_l1(key).await;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use errors::AppError;

    struct MockCache {
        should_fail: bool,
    }

    #[async_trait]
    impl CachePort for MockCache {
        async fn get(&self, _key: &str) -> AppResult<Option<String>> {
            if self.should_fail {
                Err(AppError::internal("L2 cache error"))
            } else {
                Ok(Some("l2_value".to_string()))
            }
        }

        async fn set(&self, _key: &str, _value: &str, _ttl: Option<Duration>) -> AppResult<()> {
            if self.should_fail {
                Err(AppError::internal("L2 cache error"))
            } else {
                Ok(())
            }
        }

        async fn delete(&self, _key: &str) -> AppResult<()> {
            Ok(())
        }

        async fn exists(&self, _key: &str) -> AppResult<bool> {
            Ok(true)
        }

        async fn expire(&self, _key: &str, _ttl: Duration) -> AppResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_l1_cache_hit() {
        let l2 = Arc::new(MockCache { should_fail: false }) as Arc<dyn CachePort>;
        let cache = MultiLayerCache::new(l2, MultiLayerCacheConfig::default());

        // 先写入
        cache.set("key1", "value1", None).await.unwrap();

        // 从 L1 读取
        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_l2_fallback_on_error() {
        let l2 = Arc::new(MockCache { should_fail: true }) as Arc<dyn CachePort>;
        let cache = MultiLayerCache::new(l2, MultiLayerCacheConfig::default());

        // 先写入 L1（L2 会失败但降级成功）
        cache.set("key1", "value1", None).await.unwrap();

        // L2 失败，但能从 L1 读取
        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_l1_l2_consistency() {
        let l2 = Arc::new(MockCache { should_fail: false }) as Arc<dyn CachePort>;
        let cache = MultiLayerCache::new(l2, MultiLayerCacheConfig::default());

        // 写入
        cache.set("key1", "value1", None).await.unwrap();

        // 删除
        cache.delete("key1").await.unwrap();

        // 验证 L1 也被删除
        let value = cache.get_from_l1("key1").await;
        assert!(value.is_none());
    }
}
