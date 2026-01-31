//! PostgreSQL 连接管理

use errors::{AppError, AppResult};
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::config::PostgresConfig;

/// 创建 PostgreSQL 连接池
pub async fn create_pool(config: &PostgresConfig) -> AppResult<PgPool> {
    let mut options = PgPoolOptions::new()
        .max_connections(config.pool_max)
        .min_connections(config.pool_min)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout);

    // 设置连接最大生命周期（防止连接泄漏）
    if let Some(max_lifetime) = config.max_lifetime {
        options = options.max_lifetime(max_lifetime);
    }

    options
        .connect(&config.connection_url())
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

    /// 从配置创建读写分离连接池
    pub async fn from_config(config: &PostgresConfig) -> AppResult<Self> {
        let write_pool = create_pool(config).await?;

        let read_pool = if config.has_read_replicas() {
            // 使用第一个只读副本创建读连接池
            // 生产环境可以考虑使用负载均衡
            let read_config = PostgresConfig::new(&config.read_replicas[0])
                .with_pool(config.pool_min, config.pool_max)
                .with_acquire_timeout(config.acquire_timeout)
                .with_idle_timeout(config.idle_timeout);

            Some(create_pool(&read_config).await?)
        } else {
            None
        };

        Ok(Self::new(write_pool, read_pool))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_status() {
        let status = PoolStatus {
            write_size: 10,
            write_idle: 7,
            write_active: 3,
            read_size: 5,
            read_idle: 4,
            read_active: 1,
        };

        assert_eq!(status.write_size, 10);
        assert_eq!(status.write_active, 3);
        assert_eq!(status.read_size, 5);
    }
}
