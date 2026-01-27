//! 登录尝试追踪服务

use cuba_common::TenantId;
use cuba_errors::AppResult;
use std::sync::Arc;

/// 登录尝试追踪服务
pub struct LoginAttemptService {
    cache: Arc<dyn LoginAttemptCache>,
}

impl LoginAttemptService {
    pub fn new(cache: Arc<dyn LoginAttemptCache>) -> Self {
        Self { cache }
    }

    /// 记录登录失败
    pub async fn record_failure(&self, username: &str, tenant_id: &TenantId) -> AppResult<()> {
        let key = Self::make_key(username, tenant_id);
        let count = self.cache.increment(&key, 900).await?; // 15分钟 TTL

        tracing::warn!(
            username = %username,
            tenant_id = %tenant_id,
            attempt_count = count,
            "Login attempt failed"
        );

        Ok(())
    }

    /// 检查是否被锁定
    pub async fn is_locked(&self, username: &str, tenant_id: &TenantId) -> AppResult<bool> {
        let key = Self::make_key(username, tenant_id);
        let count = self.cache.get(&key).await?;

        Ok(count >= 5)
    }

    /// 获取失败次数
    pub async fn get_failure_count(&self, username: &str, tenant_id: &TenantId) -> AppResult<i32> {
        let key = Self::make_key(username, tenant_id);
        self.cache.get(&key).await
    }

    /// 检查是否需要验证码
    pub async fn requires_captcha(&self, username: &str, tenant_id: &TenantId) -> AppResult<bool> {
        let count = self.get_failure_count(username, tenant_id).await?;
        Ok(count >= 3)
    }

    /// 清除失败记录（登录成功后）
    pub async fn clear_failures(&self, username: &str, tenant_id: &TenantId) -> AppResult<()> {
        let key = Self::make_key(username, tenant_id);
        self.cache.delete(&key).await?;

        tracing::info!(
            username = %username,
            tenant_id = %tenant_id,
            "Login attempt counter cleared"
        );

        Ok(())
    }

    /// 获取剩余锁定时间（秒）
    pub async fn get_lock_remaining_seconds(
        &self,
        username: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<i64>> {
        let key = Self::make_key(username, tenant_id);
        self.cache.ttl(&key).await
    }

    fn make_key(username: &str, tenant_id: &TenantId) -> String {
        format!("login:failed:{}:{}", username, tenant_id.0)
    }
}

/// 登录尝试缓存接口
#[async_trait::async_trait]
pub trait LoginAttemptCache: Send + Sync {
    /// 增加计数器，返回新值
    async fn increment(&self, key: &str, ttl_seconds: i64) -> AppResult<i32>;

    /// 获取计数器值
    async fn get(&self, key: &str) -> AppResult<i32>;

    /// 删除计数器
    async fn delete(&self, key: &str) -> AppResult<()>;

    /// 获取 TTL（秒）
    async fn ttl(&self, key: &str) -> AppResult<Option<i64>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockCache {
        data: Mutex<HashMap<String, (i32, i64)>>, // (value, expire_at)
    }

    impl MockCache {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl LoginAttemptCache for MockCache {
        async fn increment(&self, key: &str, ttl_seconds: i64) -> AppResult<i32> {
            let mut data = self.data.lock().unwrap();
            let expire_at = chrono::Utc::now().timestamp() + ttl_seconds;
            let entry = data.entry(key.to_string()).or_insert((0, expire_at));
            entry.0 += 1;
            Ok(entry.0)
        }

        async fn get(&self, key: &str) -> AppResult<i32> {
            let data = self.data.lock().unwrap();
            Ok(data.get(key).map(|(v, _)| *v).unwrap_or(0))
        }

        async fn delete(&self, key: &str) -> AppResult<()> {
            let mut data = self.data.lock().unwrap();
            data.remove(key);
            Ok(())
        }

        async fn ttl(&self, key: &str) -> AppResult<Option<i64>> {
            let data = self.data.lock().unwrap();
            Ok(data.get(key).map(|(_, expire_at)| {
                let now = chrono::Utc::now().timestamp();
                (*expire_at - now).max(0)
            }))
        }
    }

    #[tokio::test]
    async fn test_record_failure() {
        let cache = Arc::new(MockCache::new());
        let service = LoginAttemptService::new(cache);
        let tenant_id = TenantId::new();

        // 记录失败
        service.record_failure("testuser", &tenant_id).await.unwrap();
        let count = service.get_failure_count("testuser", &tenant_id).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_is_locked_after_5_failures() {
        let cache = Arc::new(MockCache::new());
        let service = LoginAttemptService::new(cache);
        let tenant_id = TenantId::new();

        // 记录5次失败
        for _ in 0..5 {
            service.record_failure("testuser", &tenant_id).await.unwrap();
        }

        // 应该被锁定
        let locked = service.is_locked("testuser", &tenant_id).await.unwrap();
        assert!(locked);
    }

    #[tokio::test]
    async fn test_requires_captcha_after_3_failures() {
        let cache = Arc::new(MockCache::new());
        let service = LoginAttemptService::new(cache);
        let tenant_id = TenantId::new();

        // 2次失败，不需要验证码
        for _ in 0..2 {
            service.record_failure("testuser", &tenant_id).await.unwrap();
        }
        assert!(!service.requires_captcha("testuser", &tenant_id).await.unwrap());

        // 3次失败，需要验证码
        service.record_failure("testuser", &tenant_id).await.unwrap();
        assert!(service.requires_captcha("testuser", &tenant_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_clear_failures() {
        let cache = Arc::new(MockCache::new());
        let service = LoginAttemptService::new(cache);
        let tenant_id = TenantId::new();

        // 记录失败
        service.record_failure("testuser", &tenant_id).await.unwrap();
        assert_eq!(service.get_failure_count("testuser", &tenant_id).await.unwrap(), 1);

        // 清除
        service.clear_failures("testuser", &tenant_id).await.unwrap();
        assert_eq!(service.get_failure_count("testuser", &tenant_id).await.unwrap(), 0);
    }
}
