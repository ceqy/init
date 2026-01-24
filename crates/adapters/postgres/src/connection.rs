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
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
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
}

/// 创建 PostgreSQL 连接池
pub async fn create_pool(config: &PostgresConfig) -> AppResult<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.connect_timeout)
        .idle_timeout(config.idle_timeout)
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
