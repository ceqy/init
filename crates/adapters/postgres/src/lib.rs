//! cuba-adapter-postgres - PostgreSQL 适配器
//!
//! 提供完整的 PostgreSQL 数据库支持，包括：
//! - 连接池管理（支持读写分离）
//! - 事务管理（支持两阶段提交）
//! - 数据库迁移
//! - 健康检查
//! - 重试机制
//! - Event Store 实现
//! - Outbox 模式实现

pub mod config;
pub mod connection;
pub mod event_store;
pub mod health;
pub mod migration;
pub mod outbox;
pub mod retry;
pub mod transaction;

pub use config::{PostgresConfig, RetryConfig, SslMode};
pub use connection::{check_connection, create_pool, PoolStatus, ReadWritePool};
pub use event_store::PostgresEventStore;
pub use health::{
    check_pool_health, get_database_stats, DatabaseStats, HealthCheckResult, HealthChecker,
    PoolHealthStatus,
};
pub use migration::{Migration, MigrationError, MigrationManager, MigrationRecord, MigrationResult};
pub use outbox::PostgresOutbox;
pub use retry::{
    is_retryable_error, is_sqlx_retryable, with_conditional_retry, with_retry, with_retry_app,
};
pub use transaction::{
    AccessMode, IsolationLevel, PreparedTransaction, Savepoint, TransactionContext,
    TransactionManager, TransactionOptions, TwoPhaseCommit,
};
