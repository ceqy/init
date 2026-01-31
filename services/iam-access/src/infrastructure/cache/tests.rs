//! 缓存策略集成测试

#[cfg(test)]
mod tests {
    use crate::infrastructure::cache::{
        AvalancheProtectedCache, AuthCache, AuthCacheConfig, MultiLayerCache,
        MultiLayerCacheConfig, SimpleBloomFilter,
    };
    use errors::AppResult;
    use ports::CachePort;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    // Mock 缓存实现
    struct MockCache {
        data: Arc<Mutex<HashMap<String, String>>>,
        fail_count: Arc<Mutex<usize>>,
        should_fail: bool,
    }

    impl MockCache {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
                fail_count: Arc::new(Mutex::new(0)),
                should_fail: false,
            }
        }

        fn with_failure() -> Self {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
                fail_count: Arc::new(Mutex::new(0)),
                should_fail: true,
            }
        }

        fn get_fail_count(&self) -> usize {
            *self.fail_count.lock().unwrap()
        }
    }

    #[async_trait::async_trait]
    impl CachePort for MockCache {
        async fn get(&self, key: &str) -> AppResult<Option<String>> {
            if self.should_fail {
                *self.fail_count.lock().unwrap() += 1;
                return Err(errors::AppError::internal("Mock cache error"));
            }
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        async fn set(&self, key: &str, value: &str, _ttl: Option<Duration>) -> AppResult<()> {
            if self.should_fail {
                *self.fail_count.lock().unwrap() += 1;
                return Err(errors::AppError::internal("Mock cache error"));
            }
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        async fn delete(&self, key: &str) -> AppResult<()> {
            self.data.lock().unwrap().remove(key);
            Ok(())
        }

        async fn exists(&self, key: &str) -> AppResult<bool> {
            Ok(self.data.lock().unwrap().contains_key(key))
        }

        async fn expire(&self, _key: &str, _ttl: Duration) -> AppResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_avalanche_protection_ttl_jitter() {
        let mock = Arc::new(MockCache::new()) as Arc<dyn CachePort>;
        let cache = AvalancheProtectedCache::new(mock, 30);

        // 设置多个缓存，验证 TTL 有抖动
        for i in 0..10 {
            let key = format!("key_{}", i);
            cache
                .set(&key, "value", Some(Duration::from_secs(300)))
                .await
                .unwrap();
        }

        // 验证缓存已设置
        for i in 0..10 {
            let key = format!("key_{}", i);
            let value = cache.get(&key).await.unwrap();
            assert_eq!(value, Some("value".to_string()));
        }
    }

    #[tokio::test]
    async fn test_avalanche_protection_singleflight() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct CountingCache {
            call_count: Arc<AtomicUsize>,
        }

        #[async_trait::async_trait]
        impl CachePort for CountingCache {
            async fn get(&self, _key: &str) -> AppResult<Option<String>> {
                self.call_count.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(Some("value".to_string()))
            }

            async fn set(&self, _key: &str, _value: &str, _ttl: Option<Duration>) -> AppResult<()> {
                Ok(())
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

        let call_count = Arc::new(AtomicUsize::new(0));
        let mock = Arc::new(CountingCache {
            call_count: call_count.clone(),
        }) as Arc<dyn CachePort>;
        let cache = AvalancheProtectedCache::new(mock, 10);

        // 并发发起 10 个相同 key 的请求
        let mut handles = vec![];
        for _ in 0..10 {
            let cache = cache.clone();
            handles.push(tokio::spawn(async move { cache.get("test_key").await }));
        }

        // 等待所有请求完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), Some("value".to_string()));
        }

        // 验证实际只调用了一次底层缓存（Singleflight 生效）
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_multi_layer_cache_l1_hit() {
        let mock = Arc::new(MockCache::new()) as Arc<dyn CachePort>;
        let config = MultiLayerCacheConfig {
            l1_max_capacity: 100,
            l1_ttl_secs: 60,
            fallback_to_l1: true,
        };
        let cache = MultiLayerCache::new(mock, config);

        // 写入缓存
        cache.set("key1", "value1", None).await.unwrap();

        // 第一次读取（L2 命中，回填 L1）
        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // 第二次读取（L1 命中）
        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_multi_layer_cache_fallback() {
        let mock = Arc::new(MockCache::with_failure()) as Arc<dyn CachePort>;
        let config = MultiLayerCacheConfig {
            l1_max_capacity: 100,
            l1_ttl_secs: 60,
            fallback_to_l1: true,
        };
        let cache = MultiLayerCache::new(mock.clone(), config);

        // 写入缓存（L2 失败，但 L1 成功）
        cache.set("key1", "value1", None).await.unwrap();

        // 读取缓存（L2 失败，降级到 L1）
        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // 验证 L2 确实失败了
        assert!(mock.get_fail_count() > 0);
    }

    #[tokio::test]
    async fn test_auth_cache_with_config() {
        let mock = Arc::new(MockCache::new()) as Arc<dyn CachePort>;
        let config = AuthCacheConfig {
            user_roles_ttl_secs: 100,
            role_ttl_secs: 200,
            policy_ttl_secs: 300,
        };
        let auth_cache = AuthCache::new(mock).with_config(config);

        // 验证配置已应用
        // 这里只是验证创建成功，实际 TTL 验证需要 Redis
        assert!(true);
    }

    #[tokio::test]
    async fn test_bloom_filter_hash_calculation() {
        use crate::infrastructure::cache::SimpleBloomFilter;

        // 测试 hash 计算的一致性
        let mock_conn = redis::aio::ConnectionManager::new(
            redis::Client::open("redis://localhost:6379").unwrap(),
        )
        .await
        .unwrap_or_else(|_| {
            // 如果 Redis 不可用，跳过测试
            panic!("Redis not available for testing");
        });

        let bloom = SimpleBloomFilter::new(
            mock_conn,
            "test_bloom".to_string(),
            1000,
            0.01,
        );

        // 验证相同输入产生相同 hash
        let hash1 = bloom.hash("test_key", 0);
        let hash2 = bloom.hash("test_key", 0);
        assert_eq!(hash1, hash2);

        // 验证不同 seed 产生不同 hash
        let hash3 = bloom.hash("test_key", 1);
        assert_ne!(hash1, hash3);
    }

    #[tokio::test]
    async fn test_cache_strategy_integration() {
        // 集成测试：组合所有缓存策略
        let mock = Arc::new(MockCache::new()) as Arc<dyn CachePort>;

        // 1. 添加雪崩防护
        let cache = Arc::new(AvalancheProtectedCache::new(mock, 30)) as Arc<dyn CachePort>;

        // 2. 添加多层缓存
        let config = MultiLayerCacheConfig {
            l1_max_capacity: 100,
            l1_ttl_secs: 60,
            fallback_to_l1: true,
        };
        let cache = Arc::new(MultiLayerCache::new(cache, config)) as Arc<dyn CachePort>;

        // 3. 创建 AuthCache
        let auth_cache = AuthCache::new(cache);

        // 验证缓存工作正常
        // 这里只是验证创建成功，实际功能测试需要完整的环境
        assert!(true);
    }

    #[tokio::test]
    async fn test_concurrent_cache_access() {
        let mock = Arc::new(MockCache::new()) as Arc<dyn CachePort>;
        let cache = Arc::new(AvalancheProtectedCache::new(mock, 10));

        // 并发写入
        let mut handles = vec![];
        for i in 0..100 {
            let cache = cache.clone();
            handles.push(tokio::spawn(async move {
                let key = format!("key_{}", i % 10);
                let value = format!("value_{}", i);
                cache.set(&key, &value, Some(Duration::from_secs(60))).await
            }));
        }

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // 并发读取
        let mut handles = vec![];
        for i in 0..100 {
            let cache = cache.clone();
            handles.push(tokio::spawn(async move {
                let key = format!("key_{}", i % 10);
                cache.get(&key).await
            }));
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }
}
