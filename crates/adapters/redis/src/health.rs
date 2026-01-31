//! Redis 健康检查模块
//!
//! 提供连接池级别的健康检查

use std::sync::Arc;
use std::time::Duration;

use errors::AppResult;
use tracing::{debug, error, info};

use crate::pool::{check_connection, RedisPool};

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
    pub pool_status: Option<crate::pool::PoolStatus>,
}

/// 健康检查器
pub struct HealthChecker {
    pool: Arc<RedisPool>,
    check_interval: Duration,
    timeout: Duration,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(pool: Arc<RedisPool>) -> Self {
        Self {
            pool,
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
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

    /// 执行健康检查
    pub async fn check(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        let conn_result = self.pool.get().await;
        let conn = match conn_result {
            Ok(c) => c,
            Err(e) => {
                self.pool.mark_unhealthy();
                return HealthCheckResult {
                    healthy: false,
                    latency_ms: None,
                    error: Some(format!("Failed to get connection: {}", e)),
                    pool_status: Some(self.pool.status()),
                };
            }
        };

        let mut conn_manager = conn.connection();
        let result = tokio::time::timeout(self.timeout, check_connection(&mut conn_manager)).await;

        match result {
            Ok(Ok(())) => {
                let latency = start.elapsed().as_millis() as u64;
                self.pool.mark_healthy();
                debug!(latency_ms = latency, "Redis health check passed");
                HealthCheckResult {
                    healthy: true,
                    latency_ms: Some(latency),
                    error: None,
                    pool_status: Some(self.pool.status()),
                }
            }
            Ok(Err(e)) => {
                self.pool.mark_unhealthy();
                error!(error = %e, "Redis health check failed");
                HealthCheckResult {
                    healthy: false,
                    latency_ms: None,
                    error: Some(e.to_string()),
                    pool_status: Some(self.pool.status()),
                }
            }
            Err(_) => {
                self.pool.mark_unhealthy();
                error!("Redis health check timed out");
                HealthCheckResult {
                    healthy: false,
                    latency_ms: None,
                    error: Some("Health check timed out".to_string()),
                    pool_status: Some(self.pool.status()),
                }
            }
        }
    }

    /// 快速健康检查
    pub async fn quick_check(&self) -> AppResult<()> {
        let conn = self.pool.get().await?;
        let mut conn_manager = conn.connection();
        check_connection(&mut conn_manager).await
    }

    /// 启动后台健康检查任务
    pub fn start_background_check(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!(
                interval_secs = self.check_interval.as_secs(),
                "Starting Redis background health checker"
            );

            loop {
                tokio::time::sleep(self.check_interval).await;

                let result = self.check().await;

                if result.healthy {
                    debug!(
                        latency_ms = result.latency_ms,
                        "Redis health check passed"
                    );
                } else {
                    error!(
                        error = ?result.error,
                        "Redis health check failed"
                    );
                }
            }
        })
    }
}

/// 简单的健康检查函数（内部使用）
async fn _check_pool_health(pool: &RedisPool) -> AppResult<()> {
    let conn = pool.get().await?;
    let mut conn_manager = conn.connection();
    check_connection(&mut conn_manager).await
}

/// 检查连接池是否健康
pub fn is_pool_healthy(pool: &RedisPool) -> bool {
    pool.is_healthy()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisConfig;

    #[test]
    fn test_health_check_result() {
        let result = HealthCheckResult {
            healthy: true,
            latency_ms: Some(10),
            error: None,
            pool_status: None,
        };
        assert!(result.healthy);
        assert_eq!(result.latency_ms, Some(10));
    }

    #[tokio::test]
    #[ignore] // 需要 Redis 实例
    async fn test_health_checker() {
        let config = RedisConfig::new("redis://127.0.0.1:6379");
        let pool = Arc::new(RedisPool::new(config).await.unwrap());
        let checker = HealthChecker::new(pool)
            .with_interval(Duration::from_secs(60))
            .with_timeout(Duration::from_secs(10));

        let result = checker.check().await;
        assert!(result.healthy);
    }
}
