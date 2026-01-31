//! ClickHouse 健康检查模块
//!
//! 提供连接池和节点级别的健康检查

use std::sync::Arc;
use std::time::Duration;

use cuba_errors::AppResult;
use tracing::{debug, error, info};

use crate::client::{ClickHousePool, check_connection};

/// 健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// 是否健康
    pub healthy: bool,
    /// 节点索引
    pub node_index: usize,
    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
}

/// 连接池健康状态
#[derive(Debug, Clone)]
pub struct PoolHealthStatus {
    /// 整体是否健康
    pub healthy: bool,
    /// 各节点健康状态
    pub nodes: Vec<HealthCheckResult>,
    /// 健康节点数
    pub healthy_count: usize,
    /// 总节点数
    pub total_count: usize,
}

impl PoolHealthStatus {
    /// 是否完全健康（所有节点都健康）
    pub fn is_fully_healthy(&self) -> bool {
        self.healthy_count == self.total_count
    }

    /// 是否部分健康（至少有一个节点健康）
    pub fn is_partially_healthy(&self) -> bool {
        self.healthy_count > 0
    }
}

/// 健康检查器
pub struct HealthChecker {
    pool: Arc<ClickHousePool>,
    check_interval: Duration,
    timeout: Duration,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(pool: Arc<ClickHousePool>) -> Self {
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

    /// 检查单个节点
    pub async fn check_node(&self, node_index: usize) -> HealthCheckResult {
        let client = match self.pool.client(node_index) {
            Some(c) => c,
            None => {
                return HealthCheckResult {
                    healthy: false,
                    node_index,
                    latency_ms: None,
                    error: Some("Node not found".to_string()),
                };
            }
        };

        let start = std::time::Instant::now();
        let result = tokio::time::timeout(self.timeout, check_connection(client)).await;

        match result {
            Ok(Ok(())) => {
                let latency = start.elapsed().as_millis() as u64;
                debug!(node_index, latency_ms = latency, "ClickHouse node healthy");
                HealthCheckResult {
                    healthy: true,
                    node_index,
                    latency_ms: Some(latency),
                    error: None,
                }
            }
            Ok(Err(e)) => {
                error!(node_index, error = %e, "ClickHouse node unhealthy");
                HealthCheckResult {
                    healthy: false,
                    node_index,
                    latency_ms: None,
                    error: Some(e.to_string()),
                }
            }
            Err(_) => {
                error!(node_index, "ClickHouse node health check timed out");
                HealthCheckResult {
                    healthy: false,
                    node_index,
                    latency_ms: None,
                    error: Some("Health check timed out".to_string()),
                }
            }
        }
    }

    /// 检查所有节点
    pub async fn check_all(&self) -> PoolHealthStatus {
        let node_count = self.pool.node_count();
        let mut nodes = Vec::with_capacity(node_count);

        for i in 0..node_count {
            let result = self.check_node(i).await;

            // 更新连接池中的健康状态
            if result.healthy {
                self.pool.mark_healthy(i);
            } else {
                self.pool.mark_unhealthy(i);
            }

            nodes.push(result);
        }

        let healthy_count = nodes.iter().filter(|n| n.healthy).count();

        PoolHealthStatus {
            healthy: healthy_count > 0,
            nodes,
            healthy_count,
            total_count: node_count,
        }
    }

    /// 快速健康检查（只检查一个节点）
    pub async fn quick_check(&self) -> AppResult<()> {
        let client = self.pool.get().await?;
        check_connection(client.client()).await
    }

    /// 启动后台健康检查任务
    pub fn start_background_check(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!(
                interval_secs = self.check_interval.as_secs(),
                "Starting ClickHouse background health checker"
            );

            loop {
                tokio::time::sleep(self.check_interval).await;

                let status = self.check_all().await;

                if status.is_fully_healthy() {
                    debug!(
                        healthy = status.healthy_count,
                        total = status.total_count,
                        "All ClickHouse nodes healthy"
                    );
                } else if status.is_partially_healthy() {
                    info!(
                        healthy = status.healthy_count,
                        total = status.total_count,
                        "Some ClickHouse nodes unhealthy"
                    );
                } else {
                    error!("All ClickHouse nodes unhealthy!");
                }
            }
        })
    }
}

/// 检查连接池是否有可用节点
pub fn has_healthy_nodes(pool: &ClickHousePool) -> bool {
    pool.status().healthy_nodes > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ClickHouseConfig;

    #[test]
    fn test_health_check_result() {
        let result = HealthCheckResult {
            healthy: true,
            node_index: 0,
            latency_ms: Some(10),
            error: None,
        };
        assert!(result.healthy);
        assert_eq!(result.latency_ms, Some(10));
    }

    #[test]
    fn test_pool_health_status() {
        let status = PoolHealthStatus {
            healthy: true,
            nodes: vec![
                HealthCheckResult {
                    healthy: true,
                    node_index: 0,
                    latency_ms: Some(10),
                    error: None,
                },
                HealthCheckResult {
                    healthy: false,
                    node_index: 1,
                    latency_ms: None,
                    error: Some("Connection refused".to_string()),
                },
            ],
            healthy_count: 1,
            total_count: 2,
        };

        assert!(status.is_partially_healthy());
        assert!(!status.is_fully_healthy());
    }

    #[test]
    fn test_health_checker_builder() {
        let config = ClickHouseConfig::new("http://localhost:8123", "test");
        let pool = Arc::new(ClickHousePool::new(config).unwrap());
        let checker = HealthChecker::new(pool)
            .with_interval(Duration::from_secs(60))
            .with_timeout(Duration::from_secs(10));

        assert_eq!(checker.check_interval, Duration::from_secs(60));
        assert_eq!(checker.timeout, Duration::from_secs(10));
    }
}
