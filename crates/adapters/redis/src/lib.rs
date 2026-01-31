//! adapter-redis - Redis 适配器
//!
//! 提供完整的 Redis 功能支持：
//! - 缓存操作
//! - 连接池管理
//! - 分布式锁
//! - 发布/订阅
//! - Redis Stream 消息队列
//! - 分布式限流
//! - 健康检查
//! - 集群支持

mod cache;
mod cluster;
mod config;
mod distributed_lock;
mod health;
mod pool;
mod pubsub;
mod rate_limiter;
mod retry;
mod stream;

pub use cache::*;
pub use cluster::*;
pub use config::*;
pub use distributed_lock::*;
pub use health::*;
pub use pool::{
    check_connection, check_pool_health, create_connection_manager, PoolStatus, PooledConnection,
    RedisPool,
};
pub use pubsub::*;
pub use rate_limiter::*;
pub use retry::*;
pub use stream::*;
