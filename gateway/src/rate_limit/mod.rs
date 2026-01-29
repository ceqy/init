//! 分级限流中间件
//!
//! 基于用户等级和接口类型的动态限流机制

// TODO: 移除此 allow，修复 rate_limit_middleware 的 trait bound 问题后启用
#![allow(unused_imports)]
#![allow(dead_code)]

pub mod classifier;
pub mod config;
pub mod limiter;
pub mod middleware;
pub mod tier;
pub mod types;

pub use classifier::EndpointClassifier;
pub use config::ConfigManager;
pub use limiter::RateLimiter;
pub use middleware::{RateLimitMiddleware, rate_limit_middleware};
pub use types::{
    EndpointType, RateLimitConfig as RateLimitRateConfig, RateLimitResult, RateLimitRule, UserTier,
};
