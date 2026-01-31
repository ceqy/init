//! 缓存策略改进示例
//!
//! 展示如何在 iam-access 服务中使用改进的缓存策略

use crate::infrastructure::cache::{
    AuthCache, AuthCacheConfig, AvalancheProtectedCache, CacheWarmer, MultiLayerCache,
    MultiLayerCacheConfig, SimpleBloomFilter,
};
use adapter_redis::RedisCache;
use errors::AppResult;
use redis::aio::ConnectionManager;
use std::sync::Arc;

/// 缓存策略配置
#[derive(Clone)]
pub struct CacheStrategyConfig {
    /// 是否启用多层缓存
    pub enable_multi_layer: bool,
    /// 是否启用雪崩防护
    pub enable_avalanche_protection: bool,
    /// 是否启用布隆过滤器
    pub enable_bloom_filter: bool,
    /// 是否启用缓存预热
    pub enable_cache_warming: bool,
    /// TTL 抖动范围（秒）
    pub jitter_range_secs: u64,
    /// 认证缓存配置
    pub auth_cache_config: AuthCacheConfig,
    /// 多层缓存配置
    pub multi_layer_config: MultiLayerCacheConfig,
}

impl Default for CacheStrategyConfig {
    fn default() -> Self {
        Self {
            enable_multi_layer: true,
            enable_avalanche_protection: true,
            enable_bloom_filter: false, // 需要 RedisBloom 模块
            enable_cache_warming: true,
            jitter_range_secs: 30, // ±15 秒抖动
            auth_cache_config: AuthCacheConfig::default(),
            multi_layer_config: MultiLayerCacheConfig::default(),
        }
    }
}

/// 创建增强的缓存实例
///
/// 根据配置组合不同的缓存策略：
/// 1. 基础 Redis 缓存
/// 2. 雪崩防护（TTL 抖动 + Singleflight）
/// 3. 多层缓存（L1 内存 + L2 Redis）
/// 4. 布隆过滤器（防止缓存穿透）
pub fn create_enhanced_cache(
    redis_conn: ConnectionManager,
    config: CacheStrategyConfig,
) -> Arc<AuthCache> {
    // 1. 基础 Redis 缓存
    let redis_cache = RedisCache::new(redis_conn.clone());
    let mut cache: Arc<dyn ports::CachePort> = Arc::new(redis_cache);

    // 2. 雪崩防护层
    if config.enable_avalanche_protection {
        tracing::info!(
            "Enabling avalanche protection with jitter range: {} seconds",
            config.jitter_range_secs
        );
        cache = Arc::new(AvalancheProtectedCache::new(
            cache,
            config.jitter_range_secs,
        ));
    }

    // 3. 多层缓存层
    if config.enable_multi_layer {
        tracing::info!(
            "Enabling multi-layer cache (L1 max: {}, TTL: {}s)",
            config.multi_layer_config.l1_max_capacity,
            config.multi_layer_config.l1_ttl_secs
        );
        cache = Arc::new(MultiLayerCache::new(cache, config.multi_layer_config));
    }

    // 4. 创建 AuthCache
    let auth_cache = AuthCache::new(cache).with_config(config.auth_cache_config);

    Arc::new(auth_cache)
}

/// 创建布隆过滤器（用于防止缓存穿透）
///
/// 使用场景：
/// - 在查询用户角色前，先检查用户是否存在
/// - 在查询策略前，先检查租户是否存在
pub fn create_bloom_filter(
    redis_conn: ConnectionManager,
    name: String,
    expected_items: u64,
    false_positive_rate: f64,
) -> SimpleBloomFilter {
    SimpleBloomFilter::new(redis_conn, name, expected_items, false_positive_rate)
}

/// 启动缓存预热
///
/// 在应用启动时预加载热点数据
pub async fn start_cache_warming(
    auth_cache: Arc<AuthCache>,
    policy_repo: Arc<dyn crate::domain::policy::PolicyRepository>,
    role_repo: Arc<dyn crate::domain::role::RoleRepository>,
    tenant_ids: Vec<common::TenantId>,
) -> AppResult<()> {
    let warmer = CacheWarmer::new(auth_cache);

    // 定义策略加载器
    let policy_loader = {
        let repo = policy_repo.clone();
        move |tenant_id: &common::TenantId| {
            let repo = repo.clone();
            let tenant_id = tenant_id.clone();
            Box::pin(async move { repo.list_active_by_tenant(&tenant_id).await })
                as std::pin::Pin<
                    Box<
                        dyn std::future::Future<
                                Output = AppResult<Vec<crate::domain::policy::Policy>>,
                            > + Send,
                    >,
                >
        }
    };

    // 定义角色加载器（加载所有租户的热点角色）
    let role_loader = {
        let _repo = role_repo.clone();
        move || {
            Box::pin(async move {
                // 这里可以实现加载热点角色的逻辑
                // 例如：加载所有租户的 "admin" 角色
                Ok(vec![])
            })
                as std::pin::Pin<
                    Box<
                        dyn std::future::Future<Output = AppResult<Vec<crate::domain::role::Role>>>
                            + Send,
                    >,
                >
        }
    };

    // 启动预热
    warmer
        .warm_all(policy_loader, role_loader, &tenant_ids)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CacheStrategyConfig::default();
        assert!(config.enable_multi_layer);
        assert!(config.enable_avalanche_protection);
        assert_eq!(config.jitter_range_secs, 30);
    }
}
