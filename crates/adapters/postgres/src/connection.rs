//! PostgreSQL 连接管理

use cuba_errors::{AppError, AppResult};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// PostgreSQL 连接池配置
#[derive(Debug, Clone)]
pub struct PostgresConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Option<Duration>,
    pub acquire_timeout: Duration,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10, // 仅作为 fallback，实际值由 config 层控制
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600), // 10 分钟
            max_lifetime: Some(Duration::from_secs(1800)), // 30 分钟，防止连接泄漏
            acquire_timeout: Duration::from_secs(30), // 获取连接超时
        }
    }
}

impl PostgresConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub fn with_min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }

    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    pub fn with_max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime = Some(lifetime);
        self
    }

    pub fn with_acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }
}

/// 创建 PostgreSQL 连接池
pub async fn create_pool(config: &PostgresConfig) -> AppResult<PgPool> {
    let mut options = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout);

    // 设置连接最大生命周期（防止连接泄漏）
    if let Some(max_lifetime) = config.max_lifetime {
        options = options.max_lifetime(max_lifetime);
    }

    options
        .connect(&config.url)
        .await
        .map_err(|e| AppError::database(format!("Failed to create pool: {}", e)))
}

/// 检查数据库连接
pub async fn check_connection(pool: &PgPool) -> AppResult<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| AppError::database(format!("Database health check failed: {}", e)))?;
    Ok(())
}

/// 读写分离连接池管理器
///
/// 支持主从架构，写操作使用主库，读操作使用从库
#[derive(Debug, Clone)]
pub struct ReadWritePool {
    /// 主库连接池（用于写操作）
    write_pool: PgPool,
    /// 从库连接池（用于读操作，可选）
    read_pool: Option<PgPool>,
}

impl ReadWritePool {
    /// 创建读写分离连接池
    pub fn new(write_pool: PgPool, read_pool: Option<PgPool>) -> Self {
        Self {
            write_pool,
            read_pool,
        }
    }

    /// 获取写连接池（主库）
    pub fn write_pool(&self) -> &PgPool {
        &self.write_pool
    }

    /// 获取读连接池（从库，如果没有配置则使用主库）
    pub fn read_pool(&self) -> &PgPool {
        self.read_pool.as_ref().unwrap_or(&self.write_pool)
    }

    /// 是否启用了读写分离
    pub fn has_read_replica(&self) -> bool {
        self.read_pool.is_some()
    }

    /// 获取连接池状态
    pub fn pool_status(&self) -> PoolStatus {
        let write_size = self.write_pool.size();
        let write_idle = self.write_pool.num_idle() as u32;

        let (read_size, read_idle) = if let Some(read_pool) = &self.read_pool {
            (read_pool.size(), read_pool.num_idle() as u32)
        } else {
            (0, 0)
        };

        PoolStatus {
            write_size,
            write_idle,
            write_active: write_size - write_idle,
            read_size,
            read_idle,
            read_active: read_size - read_idle,
        }
    }
}

/// 连接池状态（支持读写分离）
#[derive(Debug, Clone)]
pub struct PoolStatus {
    /// 写连接池大小
    pub write_size: u32,
    /// 写连接池空闲连接数
    pub write_idle: u32,
    /// 写连接池活跃连接数
    pub write_active: u32,
    /// 读连接池大小
    pub read_size: u32,
    /// 读连接池空闲连接数
    pub read_idle: u32,
    /// 读连接池活跃连接数
    pub read_active: u32,
}
