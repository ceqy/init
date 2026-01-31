//! PostgreSQL 健康检查模块
//!
//! 提供连接池级别的健康检查

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::{debug, error, info};

/// 健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// 是否健康
    pub healthy: bool,
    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
    /// 连接池状态
    pub pool_status: Option<PoolHealthStatus>,
    /// 数据库版本
    pub db_version: Option<String>,
    /// 复制延迟（毫秒，仅对副本有效）
    pub replication_lag_ms: Option<i64>,
}

/// 连接池健康状态
#[derive(Debug, Clone)]
pub struct PoolHealthStatus {
    /// 连接池大小
    pub size: u32,
    /// 空闲连接数
    pub idle: u32,
    /// 活跃连接数
    pub active: u32,
}

/// 健康检查器
pub struct HealthChecker {
    pool: PgPool,
    healthy: Arc<AtomicBool>,
    check_interval: Duration,
    timeout: Duration,
    check_replication: bool,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            healthy: Arc::new(AtomicBool::new(true)),
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            check_replication: false,
        }
    }

    /// 设置检查间隔
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 启用复制延迟检查
    pub fn with_replication_check(mut self, enabled: bool) -> Self {
        self.check_replication = enabled;
        self
    }

    /// 标记为健康
    pub fn mark_healthy(&self) {
        self.healthy.store(true, Ordering::SeqCst);
    }

    /// 标记为不健康
    pub fn mark_unhealthy(&self) {
        self.healthy.store(false, Ordering::SeqCst);
    }

    /// 检查是否健康
    pub fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::SeqCst)
    }

    /// 获取连接池状态
    pub fn pool_status(&self) -> PoolHealthStatus {
        let size = self.pool.size();
        let idle = self.pool.num_idle() as u32;
        PoolHealthStatus {
            size,
            idle,
            active: size.saturating_sub(idle),
        }
    }

    /// 执行健康检查
    pub async fn check(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        // 基本连接检查
        let basic_check = tokio::time::timeout(
            self.timeout,
            self.basic_health_check(),
        )
        .await;

        match basic_check {
            Ok(Ok(db_version)) => {
                let latency = start.elapsed().as_millis() as u64;

                // 可选的复制延迟检查
                let replication_lag = if self.check_replication {
                    self.check_replication_lag().await.ok()
                } else {
                    None
                };

                self.mark_healthy();
                debug!(latency_ms = latency, "PostgreSQL health check passed");

                HealthCheckResult {
                    healthy: true,
                    latency_ms: Some(latency),
                    error: None,
                    pool_status: Some(self.pool_status()),
                    db_version: Some(db_version),
                    replication_lag_ms: replication_lag,
                }
            }
            Ok(Err(e)) => {
                self.mark_unhealthy();
                error!(error = %e, "PostgreSQL health check failed");
                HealthCheckResult {
                    healthy: false,
                    latency_ms: None,
                    error: Some(e.to_string()),
                    pool_status: Some(self.pool_status()),
                    db_version: None,
                    replication_lag_ms: None,
                }
            }
            Err(_) => {
                self.mark_unhealthy();
                error!("PostgreSQL health check timed out");
                HealthCheckResult {
                    healthy: false,
                    latency_ms: None,
                    error: Some("Health check timed out".to_string()),
                    pool_status: Some(self.pool_status()),
                    db_version: None,
                    replication_lag_ms: None,
                }
            }
        }
    }

    /// 基本健康检查
    async fn basic_health_check(&self) -> AppResult<String> {
        let row: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Health check failed: {}", e)))?;

        Ok(row.0)
    }

    /// 检查复制延迟
    async fn check_replication_lag(&self) -> AppResult<i64> {
        // 检查是否为副本
        let is_replica: (bool,) = sqlx::query_as("SELECT pg_is_in_recovery()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to check recovery status: {}", e)))?;

        if !is_replica.0 {
            return Ok(0);
        }

        // 获取复制延迟
        let lag: Option<(Option<i64>,)> = sqlx::query_as(
            r#"
            SELECT EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp()))::bigint * 1000
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to get replication lag: {}", e)))?;

        Ok(lag.and_then(|(l,)| l).unwrap_or(0))
    }

    /// 快速健康检查
    pub async fn quick_check(&self) -> AppResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Quick health check failed: {}", e)))?;
        Ok(())
    }

    /// 启动后台健康检查任务
    pub fn start_background_check(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!(
                interval_secs = self.check_interval.as_secs(),
                "Starting PostgreSQL background health checker"
            );

            loop {
                tokio::time::sleep(self.check_interval).await;

                let result = self.check().await;

                if result.healthy {
                    debug!(
                        latency_ms = result.latency_ms,
                        replication_lag_ms = result.replication_lag_ms,
                        "PostgreSQL health check passed"
                    );
                } else {
                    error!(
                        error = ?result.error,
                        "PostgreSQL health check failed"
                    );
                }
            }
        })
    }
}

/// 检查连接池是否健康
pub async fn check_pool_health(pool: &PgPool) -> AppResult<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| AppError::database(format!("Pool health check failed: {}", e)))?;
    Ok(())
}

/// 获取数据库统计信息
pub async fn get_database_stats(pool: &PgPool) -> AppResult<DatabaseStats> {
    let row: (i64, i64, i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT
            (SELECT count(*) FROM pg_stat_activity WHERE state = 'active') as active_connections,
            (SELECT count(*) FROM pg_stat_activity WHERE state = 'idle') as idle_connections,
            (SELECT count(*) FROM pg_stat_activity) as total_connections,
            (SELECT xact_commit FROM pg_stat_database WHERE datname = current_database()) as commits,
            (SELECT xact_rollback FROM pg_stat_database WHERE datname = current_database()) as rollbacks
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::database(format!("Failed to get database stats: {}", e)))?;

    Ok(DatabaseStats {
        active_connections: row.0,
        idle_connections: row.1,
        total_connections: row.2,
        commits: row.3,
        rollbacks: row.4,
    })
}

/// 数据库统计信息
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// 活跃连接数
    pub active_connections: i64,
    /// 空闲连接数
    pub idle_connections: i64,
    /// 总连接数
    pub total_connections: i64,
    /// 提交事务数
    pub commits: i64,
    /// 回滚事务数
    pub rollbacks: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_result() {
        let result = HealthCheckResult {
            healthy: true,
            latency_ms: Some(10),
            error: None,
            pool_status: None,
            db_version: Some("PostgreSQL 15.0".to_string()),
            replication_lag_ms: None,
        };
        assert!(result.healthy);
        assert_eq!(result.latency_ms, Some(10));
    }

    #[test]
    fn test_pool_health_status() {
        let status = PoolHealthStatus {
            size: 10,
            idle: 7,
            active: 3,
        };
        assert_eq!(status.size, 10);
        assert_eq!(status.idle, 7);
        assert_eq!(status.active, 3);
    }
}
