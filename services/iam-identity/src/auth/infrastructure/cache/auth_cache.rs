//! 认证缓存实现
//!
//! 提供 Token 黑名单和用户信息缓存

use async_trait::async_trait;
use cuba_errors::AppResult;
use cuba_ports::CachePort;
use std::sync::Arc;
use std::time::Duration;

use crate::shared::domain::entities::User;

/// Token 黑名单 key 前缀
const TOKEN_BLACKLIST_PREFIX: &str = "auth:blacklist:";

/// 用户缓存 key 前缀
const USER_CACHE_PREFIX: &str = "auth:user:";

/// 用户缓存 TTL（5 分钟）
const USER_CACHE_TTL_SECS: u64 = 300;

/// 认证缓存 trait
#[async_trait]
pub trait AuthCache: Send + Sync {
    /// 将 Token 加入黑名单
    async fn blacklist_token(&self, jti: &str, ttl_secs: u64) -> AppResult<()>;

    /// 检查 Token 是否在黑名单中
    async fn is_token_blacklisted(&self, jti: &str) -> AppResult<bool>;

    /// 缓存用户信息
    async fn cache_user(&self, user: &User) -> AppResult<()>;

    /// 获取缓存的用户信息
    async fn get_cached_user(&self, user_id: &str) -> AppResult<Option<User>>;

    /// 删除用户缓存
    async fn invalidate_user_cache(&self, user_id: &str) -> AppResult<()>;

    /// 将用户的所有 Token 加入黑名单（用于密码修改等场景）
    async fn blacklist_user_tokens(&self, user_id: &str, ttl_secs: u64) -> AppResult<()>;

    /// 检查用户的 Token 是否被全部撤销
    async fn is_user_tokens_blacklisted(&self, user_id: &str) -> AppResult<bool>;
}

/// Redis 认证缓存实现
pub struct RedisAuthCache {
    cache: Arc<dyn CachePort>,
}

impl RedisAuthCache {
    pub fn new(cache: Arc<dyn CachePort>) -> Self {
        Self { cache }
    }

    fn blacklist_key(jti: &str) -> String {
        format!("{}{}", TOKEN_BLACKLIST_PREFIX, jti)
    }

    fn user_cache_key(user_id: &str) -> String {
        format!("{}{}", USER_CACHE_PREFIX, user_id)
    }

    fn user_blacklist_key(user_id: &str) -> String {
        format!("{}user:{}", TOKEN_BLACKLIST_PREFIX, user_id)
    }
}

#[async_trait]
impl AuthCache for RedisAuthCache {
    async fn blacklist_token(&self, jti: &str, ttl_secs: u64) -> AppResult<()> {
        let key = Self::blacklist_key(jti);
        self.cache
            .set(&key, "1", Some(Duration::from_secs(ttl_secs)))
            .await
    }

    async fn is_token_blacklisted(&self, jti: &str) -> AppResult<bool> {
        let key = Self::blacklist_key(jti);
        self.cache.exists(&key).await
    }

    async fn cache_user(&self, user: &User) -> AppResult<()> {
        let key = Self::user_cache_key(&user.id.0.to_string());
        let value = serde_json::to_string(user)
            .map_err(|e| cuba_errors::AppError::internal(format!("Failed to serialize user: {}", e)))?;
        self.cache
            .set(&key, &value, Some(Duration::from_secs(USER_CACHE_TTL_SECS)))
            .await
    }

    async fn get_cached_user(&self, user_id: &str) -> AppResult<Option<User>> {
        let key = Self::user_cache_key(user_id);
        match self.cache.get(&key).await? {
            Some(value) => {
                let user: User = serde_json::from_str(&value).map_err(|e| {
                    cuba_errors::AppError::internal(format!("Failed to deserialize user: {}", e))
                })?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn invalidate_user_cache(&self, user_id: &str) -> AppResult<()> {
        let key = Self::user_cache_key(user_id);
        self.cache.delete(&key).await
    }

    async fn blacklist_user_tokens(&self, user_id: &str, ttl_secs: u64) -> AppResult<()> {
        let key = Self::user_blacklist_key(user_id);
        // 存储当前时间戳，用于判断 Token 是否在此时间之前签发
        let timestamp = chrono::Utc::now().timestamp().to_string();
        self.cache
            .set(&key, &timestamp, Some(Duration::from_secs(ttl_secs)))
            .await
    }

    async fn is_user_tokens_blacklisted(&self, user_id: &str) -> AppResult<bool> {
        let key = Self::user_blacklist_key(user_id);
        self.cache.exists(&key).await
    }
}

/// 空操作缓存实现（用于测试或禁用缓存场景）
pub struct NoOpAuthCache;

#[async_trait]
impl AuthCache for NoOpAuthCache {
    async fn blacklist_token(&self, _jti: &str, _ttl_secs: u64) -> AppResult<()> {
        Ok(())
    }

    async fn is_token_blacklisted(&self, _jti: &str) -> AppResult<bool> {
        Ok(false)
    }

    async fn cache_user(&self, _user: &User) -> AppResult<()> {
        Ok(())
    }

    async fn get_cached_user(&self, _user_id: &str) -> AppResult<Option<User>> {
        Ok(None)
    }

    async fn invalidate_user_cache(&self, _user_id: &str) -> AppResult<()> {
        Ok(())
    }

    async fn blacklist_user_tokens(&self, _user_id: &str, _ttl_secs: u64) -> AppResult<()> {
        Ok(())
    }

    async fn is_user_tokens_blacklisted(&self, _user_id: &str) -> AppResult<bool> {
        Ok(false)
    }
}
