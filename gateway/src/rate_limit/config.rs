//! 配置管理器
//!
//! 从 Redis 加载限流配置，支持本地缓存

use crate::rate_limit::types::{RateLimitConfig, RateLimitRule};
use redis::aio::ConnectionManager;
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Redis 配置键
const CONFIG_KEY: &str = "rate_limit:config:v1";

/// 配置缓存 TTL（秒）
const CACHE_TTL: u64 = 60;

/// 配置管理器
#[derive(Clone)]
#[cfg_attr(test, allow(dead_code))]
pub struct ConfigManager {
    /// Redis 连接管理器
    pub redis_conn: Arc<ConnectionManager>,
    /// 本地缓存（带 TTL）
    pub cache: Arc<RwLock<ConfigCache>>,
}

/// 配置缓存
pub struct ConfigCache {
    pub config: RateLimitConfig,
    pub expires_at: u64,
}

impl ConfigCache {
    pub fn new(config: RateLimitConfig, ttl_secs: u64) -> Self {
        let expires_at = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs())
            + ttl_secs;

        Self { config, expires_at }
    }

    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.expires_at
    }
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub async fn new(redis_conn: ConnectionManager) -> Self {
        // 初始加载配置
        let config = Self::load_config_from_redis(&redis_conn)
            .await
            .unwrap_or_else(|e| {
                warn!(error = %e, "Failed to load config from Redis, using defaults");
                RateLimitConfig::default()
            });

        Self {
            redis_conn: Arc::new(redis_conn),
            cache: Arc::new(RwLock::new(ConfigCache::new(config, CACHE_TTL))),
        }
    }

    /// 获取当前配置
    ///
    /// 首先检查本地缓存，如果过期则从 Redis 重新加载
    pub async fn get_config(&self) -> RateLimitConfig {
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if !cache.is_expired() {
                return cache.config.clone();
            }
        }

        // 缓存过期，重新加载
        debug!("Config cache expired, reloading from Redis");
        let new_config = self.reload_config().await;
        new_config
    }

    /// 强制重新加载配置
    pub async fn reload_config(&self) -> RateLimitConfig {
        let config = match Self::load_config_from_redis(&self.redis_conn).await {
            Ok(c) => c,
            Err(e) => {
                warn!(error = %e, "Failed to reload config from Redis, using cached/defaults");
                // 返回当前缓存配置或默认值
                let cache = self.cache.read().await;
                cache.config.clone()
            }
        };

        // 更新缓存
        let mut cache = self.cache.write().await;
        *cache = ConfigCache::new(config.clone(), CACHE_TTL);

        debug!("Reloaded rate limit config from Redis");
        config
    }

    /// 获取指定的限流规则
    ///
    /// # 参数
    /// - `tier`: 用户等级
    /// - `endpoint_type`: 接口类型
    ///
    /// # 优先级
    /// 1. 匹配 "tier:endpoint_type" 的规则
    /// 2. 匹配 "tier:*" 的规则
    /// 3. 匹配 "*:endpoint_type" 的规则
    /// 4. 使用默认规则
    pub async fn get_rule(&self, tier: &str, endpoint_type: &str) -> RateLimitRule {
        let config = self.get_config().await;
        let rule_key = format!("{}:{}", tier, endpoint_type);

        // 尝试精确匹配
        if let Some(rule) = config.rules.get(&rule_key) {
            debug!(key = %rule_key, "Found exact rule match");
            return rule.clone();
        }

        // 尝试匹配 tier 规则 (tier:*)
        let tier_key = format!("{}:*", tier);
        if let Some(rule) = config.rules.get(&tier_key) {
            debug!(key = %tier_key, "Found tier wildcard match");
            return rule.clone();
        }

        // 尝试匹配 endpoint 规则 (*:endpoint_type)
        let endpoint_key = format!("*:{}", endpoint_type);
        if let Some(rule) = config.rules.get(&endpoint_key) {
            debug!(key = %endpoint_key, "Found endpoint wildcard match");
            return rule.clone();
        }

        debug!(tier, endpoint_type, "Using default rule");
        config.default_rule.clone()
    }

    /// 检查是否启用限流
    pub async fn is_enabled(&self) -> bool {
        let config = self.get_config().await;
        config.enabled
    }

    /// 获取 Redis 键前缀
    pub async fn get_key_prefix(&self) -> String {
        let config = self.get_config().await;
        config.redis_key_prefix
    }

    /// 从 Redis 加载配置（静态方法）
    async fn load_config_from_redis(
        redis_conn: &ConnectionManager,
    ) -> Result<RateLimitConfig, String> {
        let mut conn = redis_conn.clone();

        // 从 Redis 获取配置
        let json: String = redis::cmd("GET")
            .arg(CONFIG_KEY)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("Redis GET failed: {}", e))?;

        if json.is_empty() {
            warn!("Rate limit config not found in Redis, using defaults");
            return Ok(RateLimitConfig::default());
        }

        // 反序列化 JSON
        let config: RateLimitConfig = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

        debug!("Loaded rate limit config from Redis");
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_cache_expiration() {
        let config = RateLimitConfig::default();
        let cache = ConfigCache::new(config, 1); // 1 秒 TTL

        assert!(!cache.is_expired());

        // 模拟时间流逝（实际测试中需要 mock SystemTime）
        // 这里只是演示结构
    }
}
