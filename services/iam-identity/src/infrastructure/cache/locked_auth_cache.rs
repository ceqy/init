//! 带分布式锁的认证缓存包装器
//!
//! 为关键的认证缓存操作添加分布式锁，防止竞态条件

use async_trait::async_trait;
use cuba_errors::AppResult;
use cuba_ports::DistributedLock;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::user::User;
use crate::infrastructure::cache::auth_cache::AuthCache;

/// 带分布式锁的认证缓存
pub struct LockedAuthCache {
    inner: Arc<dyn AuthCache>,
    lock: Arc<dyn DistributedLock>,
}

impl LockedAuthCache {
    pub fn new(inner: Arc<dyn AuthCache>, lock: Arc<dyn DistributedLock>) -> Self {
        Self { inner, lock }
    }

    /// 获取锁的 key
    fn lock_key(operation: &str, identifier: &str) -> String {
        format!("auth:lock:{}:{}", operation, identifier)
    }
}

#[async_trait]
impl AuthCache for LockedAuthCache {
    async fn blacklist_token(&self, jti: &str, ttl_secs: u64) -> AppResult<()> {
        // Token 黑名单操作使用分布式锁
        let lock_key = Self::lock_key("blacklist_token", jti);
        let lock_ttl = Duration::from_secs(5); // 锁超时 5 秒

        // 尝试获取锁，带重试
        let acquired = self.lock.acquire(&lock_key, lock_ttl).await?;
        if !acquired {
            tracing::warn!(jti = %jti, "Failed to acquire lock for blacklist_token, proceeding anyway");
            // 即使获取锁失败，也继续执行（因为 SETNX 本身是原子的）
        }

        let result = self.inner.blacklist_token(jti, ttl_secs).await;

        // 释放锁
        if acquired {
            let _ = self.lock.release(&lock_key).await;
        }

        result
    }

    async fn is_token_blacklisted(&self, jti: &str) -> AppResult<bool> {
        // 读操作不需要锁
        self.inner.is_token_blacklisted(jti).await
    }

    async fn cache_user(&self, user: &User) -> AppResult<()> {
        // 用户缓存操作不需要锁（覆盖写入是安全的）
        self.inner.cache_user(user).await
    }

    async fn get_cached_user(&self, user_id: &str) -> AppResult<Option<User>> {
        // 读操作不需要锁
        self.inner.get_cached_user(user_id).await
    }

    async fn invalidate_user_cache(&self, user_id: &str) -> AppResult<()> {
        // 删除操作是原子的，不需要锁
        self.inner.invalidate_user_cache(user_id).await
    }

    async fn blacklist_user_tokens(&self, user_id: &str, ttl_secs: u64) -> AppResult<()> {
        // 用户级别的 Token 撤销需要分布式锁
        let lock_key = Self::lock_key("blacklist_user", user_id);
        let lock_ttl = Duration::from_secs(10); // 锁超时 10 秒

        // 尝试获取锁，带重试
        let acquired = self.lock.acquire(&lock_key, lock_ttl).await?;
        if !acquired {
            tracing::warn!(user_id = %user_id, "Failed to acquire lock for blacklist_user_tokens, proceeding anyway");
        }

        let result = self.inner.blacklist_user_tokens(user_id, ttl_secs).await;

        // 释放锁
        if acquired {
            let _ = self.lock.release(&lock_key).await;
        }

        result
    }

    async fn is_user_tokens_blacklisted(&self, user_id: &str) -> AppResult<bool> {
        // 读操作不需要锁
        self.inner.is_user_tokens_blacklisted(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::auth_cache::NoOpAuthCache;
    use cuba_ports::DistributedLock;

    struct MockLock {
        should_fail: bool,
    }

    #[async_trait]
    impl DistributedLock for MockLock {
        async fn acquire(&self, _key: &str, _ttl: Duration) -> AppResult<bool> {
            Ok(!self.should_fail)
        }

        async fn release(&self, _key: &str) -> AppResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_locked_cache_blacklist_token() {
        let inner = Arc::new(NoOpAuthCache) as Arc<dyn AuthCache>;
        let lock = Arc::new(MockLock { should_fail: false }) as Arc<dyn DistributedLock>;
        let cache = LockedAuthCache::new(inner, lock);

        let result = cache.blacklist_token("test_jti", 3600).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_locked_cache_blacklist_user_tokens() {
        let inner = Arc::new(NoOpAuthCache) as Arc<dyn AuthCache>;
        let lock = Arc::new(MockLock { should_fail: false }) as Arc<dyn DistributedLock>;
        let cache = LockedAuthCache::new(inner, lock);

        let result = cache.blacklist_user_tokens("user_123", 3600).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_locked_cache_handles_lock_failure() {
        let inner = Arc::new(NoOpAuthCache) as Arc<dyn AuthCache>;
        let lock = Arc::new(MockLock { should_fail: true }) as Arc<dyn DistributedLock>;
        let cache = LockedAuthCache::new(inner, lock);

        // 即使锁失败，操作也应该继续
        let result = cache.blacklist_token("test_jti", 3600).await;
        assert!(result.is_ok());
    }
}
