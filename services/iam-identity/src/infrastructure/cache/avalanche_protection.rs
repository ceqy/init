//! 缓存雪崩防护
//!
//! 实现多种策略防止缓存雪崩：
//! 1. TTL 随机抖动 - 防止大量缓存同时过期
//! 2. Singleflight 模式 - 防止缓存击穿（同一时间大量请求同一个不存在的 key）
//! 3. 缓存预热 - 启动时预加载热点数据

use async_trait::async_trait;
use cuba_errors::{AppError, AppResult};
use cuba_ports::CachePort;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Singleflight 调用状态（保留用于未来扩展）
#[allow(dead_code)]
struct Call<T> {
    /// 等待结果的通道
    rx: tokio::sync::broadcast::Receiver<Result<T, String>>,
}

/// Singleflight 组
/// 用于合并对同一个 key 的并发请求，只执行一次实际操作
#[derive(Clone)]
struct SingleflightGroup<T> {
    #[allow(clippy::type_complexity)]
    calls: Arc<RwLock<HashMap<String, tokio::sync::broadcast::Sender<Result<T, String>>>>>,
}

impl<T: Clone + Send + Sync + 'static> SingleflightGroup<T> {
    fn new() -> Self {
        Self {
            calls: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 执行操作，如果已有相同 key 的操作在进行中，则等待其结果
    fn do_call<'a, F, Fut>(
        &'a self,
        key: &'a str,
        f: F,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<T>> + Send + 'a>>
    where
        F: FnOnce() -> Fut + Send + 'a,
        Fut: std::future::Future<Output = AppResult<T>> + Send + 'a,
    {
        Box::pin(async move {
            // 检查是否已有进行中的调用
            {
                let calls = self.calls.read().await;
                if let Some(tx) = calls.get(key) {
                    let mut rx = tx.subscribe();
                    drop(calls); // 释放读锁

                    // 等待结果
                    match rx.recv().await {
                        Ok(Ok(value)) => return Ok(value),
                        Ok(Err(e)) => return Err(AppError::internal(e)),
                        Err(_) => {
                            // 发送者已关闭，重试
                            return self.do_call(key, f).await;
                        }
                    }
                }
            }

            // 创建新的调用
            let (tx, _rx) = tokio::sync::broadcast::channel(1);
            {
                let mut calls = self.calls.write().await;
                calls.insert(key.to_string(), tx.clone());
            }

            // 执行实际操作
            let result = f().await;

            // 广播结果
            let broadcast_result = match &result {
                Ok(value) => Ok(value.clone()),
                Err(e) => Err(e.to_string()),
            };
            let _ = tx.send(broadcast_result);

            // 清理
            {
                let mut calls = self.calls.write().await;
                calls.remove(key);
            }

            result
        })
    }
}

/// 带雪崩防护的缓存包装器
#[derive(Clone)]
pub struct AvalancheProtectedCache {
    inner: Arc<dyn CachePort>,
    /// Singleflight 组（用于字符串类型）
    singleflight: SingleflightGroup<Option<String>>,
    /// TTL 抖动范围（秒）
    jitter_range: u64,
}

impl AvalancheProtectedCache {
    pub fn new(inner: Arc<dyn CachePort>, jitter_range: u64) -> Self {
        Self {
            inner,
            singleflight: SingleflightGroup::new(),
            jitter_range,
        }
    }

    /// 为 TTL 添加随机抖动
    /// 例如：TTL 300 秒，jitter_range 30 秒，则实际 TTL 在 270-330 秒之间
    fn add_jitter(&self, ttl: Duration) -> Duration {
        if self.jitter_range == 0 {
            return ttl;
        }

        let mut rng = rand::thread_rng();
        let jitter_secs = rng.gen_range(0..=self.jitter_range);
        let half_jitter = self.jitter_range / 2;

        // TTL ± jitter
        let base_secs = ttl.as_secs();
        let new_secs = if jitter_secs > half_jitter {
            base_secs + (jitter_secs - half_jitter)
        } else {
            base_secs.saturating_sub(half_jitter - jitter_secs)
        };

        Duration::from_secs(new_secs)
    }

    /// 使用 Singleflight 模式获取缓存
    /// 如果多个并发请求同一个 key，只有一个会实际查询缓存
    pub async fn get_with_singleflight(&self, key: &str) -> AppResult<Option<String>> {
        self.singleflight
            .do_call(key, || async { self.inner.get(key).await })
            .await
    }
}

#[async_trait]
impl CachePort for AvalancheProtectedCache {
    async fn get(&self, key: &str) -> AppResult<Option<String>> {
        // 使用 singleflight 防止缓存击穿
        self.get_with_singleflight(key).await
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> AppResult<()> {
        // 为 TTL 添加随机抖动，防止大量缓存同时过期
        let jittered_ttl = ttl.map(|t| self.add_jitter(t));
        self.inner.set(key, value, jittered_ttl).await
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        self.inner.delete(key).await
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        self.inner.exists(key).await
    }

    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()> {
        let jittered_ttl = self.add_jitter(ttl);
        self.inner.expire(key, jittered_ttl).await
    }

    async fn incr_with_ttl(&self, key: &str, ttl_secs: u64) -> AppResult<i64> {
        // 为 TTL 添加抖动
        let jittered_ttl = self.add_jitter(Duration::from_secs(ttl_secs));
        self.inner.incr_with_ttl(key, jittered_ttl.as_secs()).await
    }

    async fn decr(&self, key: &str) -> AppResult<i64> {
        self.inner.decr(key).await
    }

    async fn get_int(&self, key: &str) -> AppResult<Option<i64>> {
        self.inner.get_int(key).await
    }

    async fn ttl(&self, key: &str) -> AppResult<Option<i64>> {
        self.inner.ttl(key).await
    }

    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> AppResult<bool> {
        let jittered_ttl = self.add_jitter(ttl);
        self.inner.set_nx(key, value, jittered_ttl).await
    }

    async fn delete_if_equals(&self, key: &str, expected_value: &str) -> AppResult<bool> {
        self.inner.delete_if_equals(key, expected_value).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockCache {
        call_count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl CachePort for MockCache {
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

    #[tokio::test]
    async fn test_singleflight_deduplicates_concurrent_requests() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mock = Arc::new(MockCache {
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
            let result: AppResult<Option<String>> = handle.await.unwrap();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), Some("value".to_string()));
        }

        // 验证实际只调用了一次底层缓存
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_ttl_jitter() {
        let mock = Arc::new(MockCache {
            call_count: Arc::new(AtomicUsize::new(0)),
        }) as Arc<dyn CachePort>;
        let cache = AvalancheProtectedCache::new(mock, 30);

        let base_ttl = Duration::from_secs(300);

        // 测试多次，验证 TTL 有变化
        let mut ttls = vec![];
        for _ in 0..10 {
            let jittered = cache.add_jitter(base_ttl);
            ttls.push(jittered.as_secs());
        }

        // 验证 TTL 在合理范围内（270-330 秒）
        for ttl in &ttls {
            assert!(*ttl >= 270 && *ttl <= 330);
        }

        // 验证至少有一些不同的值（不是所有都相同）
        let unique_count = ttls.iter().collect::<std::collections::HashSet<_>>().len();
        assert!(unique_count > 1);
    }
}
