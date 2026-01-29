//! 在 main.rs 中集成改进的缓存策略
//!
//! 这个文件展示了如何替换原有的缓存初始化代码

// ============ 原代码 ============
/*
// 初始化缓存
use infrastructure::cache::AuthCache;
let redis_cache = infra.redis_cache();
let auth_cache = Arc::new(AuthCache::new(Arc::new(redis_cache)));
*/

// ============ 新代码（方式 1：使用策略组合 - 推荐）============
use infrastructure::cache::{create_enhanced_cache, CacheStrategyConfig, AuthCacheConfig};

// 1. 创建缓存配置
let cache_config = CacheStrategyConfig {
    enable_multi_layer: true,           // 启用多层缓存
    enable_avalanche_protection: true,  // 启用雪崩防护
    enable_bloom_filter: false,         // 布隆过滤器（需要 RedisBloom）
    enable_cache_warming: true,         // 启用缓存预热
    jitter_range_secs: 30,              // TTL 抖动范围：±15 秒
    auth_cache_config: AuthCacheConfig {
        user_roles_ttl_secs: 300,       // 用户角色 5 分钟
        role_ttl_secs: 600,             // 角色 10 分钟
        policy_ttl_secs: 600,           // 策略 10 分钟
    },
    ..Default::default()
};

// 2. 创建增强的缓存
let redis_conn = infra.redis_connection_manager();
let auth_cache = create_enhanced_cache(redis_conn, cache_config);

// 3. 可选：启动缓存预热（后台任务）
if cache_config.enable_cache_warming {
    use infrastructure::cache::start_cache_warming;

    tokio::spawn({
        let auth_cache = auth_cache.clone();
        let policy_repo = policy_repo.clone();
        let role_repo = role_repo.clone();

        async move {
            // 获取需要预热的租户列表
            // 这里可以从配置文件或数据库读取
            let tenant_ids = vec![
                // TenantId::from_str("tenant-1").unwrap(),
                // TenantId::from_str("tenant-2").unwrap(),
            ];

            if let Err(e) = start_cache_warming(
                auth_cache,
                policy_repo,
                role_repo,
                tenant_ids,
            ).await {
                tracing::warn!("Cache warming failed: {}", e);
            } else {
                tracing::info!("Cache warming completed successfully");
            }
        }
    });
}

// ============ 新代码（方式 2：手动组合）============
/*
use infrastructure::cache::{
    AvalancheProtectedCache, MultiLayerCache, MultiLayerCacheConfig,
    AuthCache, AuthCacheConfig,
};
use cuba_adapter_redis::RedisCache;
use cuba_ports::CachePort;

// 1. 基础 Redis 缓存
let redis_conn = infra.redis_connection_manager();
let redis_cache = RedisCache::new(redis_conn);
let mut cache: Arc<dyn CachePort> = Arc::new(redis_cache);

// 2. 添加雪崩防护层
cache = Arc::new(AvalancheProtectedCache::new(
    cache,
    30, // TTL 抖动范围：±15 秒
));

// 3. 添加多层缓存层
let multi_layer_config = MultiLayerCacheConfig {
    l1_max_capacity: 10_000,  // L1 最大 1 万条
    l1_ttl_secs: 60,          // L1 缓存 1 分钟
    fallback_to_l1: true,     // 启用降级
};
cache = Arc::new(MultiLayerCache::new(cache, multi_layer_config));

// 4. 创建 AuthCache
let auth_cache_config = AuthCacheConfig {
    user_roles_ttl_secs: 300,  // 用户角色 5 分钟
    role_ttl_secs: 600,         // 角色 10 分钟
    policy_ttl_secs: 600,       // 策略 10 分钟
};
let auth_cache = Arc::new(
    AuthCache::new(cache).with_config(auth_cache_config)
);
*/

// ============ 新代码（方式 3：最小改动）============
/*
// 只启用雪崩防护，其他保持不变
use infrastructure::cache::{AvalancheProtectedCache, AuthCache};
use cuba_adapter_redis::RedisCache;

let redis_conn = infra.redis_connection_manager();
let redis_cache = RedisCache::new(redis_conn);

// 添加雪崩防护
let protected_cache = Arc::new(AvalancheProtectedCache::new(
    Arc::new(redis_cache),
    30, // TTL 抖动范围
));

let auth_cache = Arc::new(AuthCache::new(protected_cache));
*/

// ============ 监控指标（可选）============
/*
// 在缓存操作中添加监控指标
use metrics;

// L1 缓存命中
metrics::counter!("cache.l1.hits", 1);

// L2 缓存命中
metrics::counter!("cache.l2.hits", 1);

// 缓存未命中
metrics::counter!("cache.misses", 1);

// Singleflight 合并
metrics::counter!("cache.singleflight.merged", 1);

// 降级次数
metrics::counter!("cache.fallback.count", 1);
*/

// ============ 布隆过滤器使用示例（可选）============
/*
use infrastructure::cache::SimpleBloomFilter;

// 创建布隆过滤器（防止缓存穿透）
let user_bloom = SimpleBloomFilter::new(
    redis_conn.clone(),
    "iam:access:user_roles_bloom".to_string(),
    100_000,  // 预期 10 万个用户
    0.01,     // 1% 误判率
);

// 在查询用户角色前先检查
async fn get_user_roles_with_bloom(
    bloom: &SimpleBloomFilter,
    cache: &AuthCache,
    tenant_id: &TenantId,
    user_id: &UserId,
) -> AppResult<Option<Vec<Role>>> {
    let bloom_key = format!("{}:{}", tenant_id, user_id);

    // 先检查布隆过滤器
    if !bloom.exists(&bloom_key).await? {
        // 一定不存在，直接返回
        tracing::debug!("Bloom filter: user not found");
        return Ok(None);
    }

    // 可能存在，查询缓存
    cache.get_user_roles(tenant_id, user_id).await
}

// 在创建用户角色时添加到布隆过滤器
async fn set_user_roles_with_bloom(
    bloom: &SimpleBloomFilter,
    cache: &AuthCache,
    tenant_id: &TenantId,
    user_id: &UserId,
    roles: &[Role],
) -> AppResult<()> {
    let bloom_key = format!("{}:", tenant_id, user_id);

    // 添加到布隆过滤器
    bloom.add(&bloom_key).await?;

    // 设置缓存
    cache.set_user_roles(tenant_id, user_id, roles).await
}
*/
