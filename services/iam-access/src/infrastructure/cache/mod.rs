//! 缓存模块
//!
//! 提供多层缓存架构和防护机制：
//! - AuthCache: 认证缓存（角色、权限、策略）
//! - AvalancheProtectedCache: 雪崩防护（TTL 抖动 + Singleflight）
//! - MultiLayerCache: 多层缓存（L1 内存 + L2 Redis）
//! - BloomFilter: 布隆过滤器（防止缓存穿透）
//! - CacheWarmer: 缓存预热
//! - Strategy: 缓存策略组合

pub mod auth_cache;
pub mod avalanche_protection;
pub mod bloom_filter;
pub mod cache_warmer;
pub mod multi_layer;
pub mod strategy;

pub use auth_cache::{AuthCache, AuthCacheConfig};
pub use avalanche_protection::AvalancheProtectedCache;
pub use bloom_filter::{BloomFilterInfo, RedisBloomFilter, SimpleBloomFilter};
pub use cache_warmer::CacheWarmer;
pub use multi_layer::{MultiLayerCache, MultiLayerCacheConfig};
pub use strategy::{
    CacheStrategyConfig, create_bloom_filter, create_enhanced_cache, start_cache_warming,
};
