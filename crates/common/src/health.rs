//! 通用健康检查模块
//!
//! 提供健康检查的通用 trait 和基础类型

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// 基础健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseHealthResult {
    /// 是否健康
    pub healthy: bool,
    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
}

impl BaseHealthResult {
    /// 创建健康结果
    pub fn healthy(latency_ms: u64) -> Self {
        Self {
            healthy: true,
            latency_ms: Some(latency_ms),
            error: None,
        }
    }

    /// 创建不健康结果
    pub fn unhealthy(error: impl Into<String>) -> Self {
        Self {
            healthy: false,
            latency_ms: None,
            error: Some(error.into()),
        }
    }

    /// 创建超时结果
    pub fn timeout() -> Self {
        Self {
            healthy: false,
            latency_ms: None,
            error: Some("Health check timed out".to_string()),
        }
    }
}

/// 带详细信息的健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult<D> {
    /// 基础结果
    pub base: BaseHealthResult,
    /// 详细信息
    pub details: Option<D>,
}

impl<D> HealthCheckResult<D> {
    /// 创建健康结果
    pub fn healthy(latency_ms: u64, details: D) -> Self {
        Self {
            base: BaseHealthResult::healthy(latency_ms),
            details: Some(details),
        }
    }

    /// 创建不健康结果
    pub fn unhealthy(error: impl Into<String>, details: Option<D>) -> Self {
        Self {
            base: BaseHealthResult::unhealthy(error),
            details,
        }
    }

    /// 创建超时结果
    pub fn timeout(details: Option<D>) -> Self {
        Self {
            base: BaseHealthResult::timeout(),
            details,
        }
    }

    /// 是否健康
    pub fn is_healthy(&self) -> bool {
        self.base.healthy
    }

    /// 获取延迟
    pub fn latency_ms(&self) -> Option<u64> {
        self.base.latency_ms
    }

    /// 获取错误信息
    pub fn error(&self) -> Option<&str> {
        self.base.error.as_deref()
    }
}

/// 健康检查 trait
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// 详细信息类型
    type Details: Send + Sync;

    /// 执行健康检查
    async fn check(&self) -> HealthCheckResult<Self::Details>;

    /// 获取组件名称
    fn component_name(&self) -> &'static str;

    /// 快速健康检查（仅检查是否可用）
    async fn quick_check(&self) -> bool {
        self.check().await.is_healthy()
    }
}

/// 健康检查器配置
#[derive(Debug, Clone)]
pub struct HealthCheckerConfig {
    /// 检查间隔
    pub check_interval: Duration,
    /// 超时时间
    pub timeout: Duration,
    /// 连续失败阈值（达到后标记为不健康）
    pub failure_threshold: u32,
    /// 连续成功阈值（达到后标记为健康）
    pub success_threshold: u32,
}

impl Default for HealthCheckerConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 1,
        }
    }
}

impl HealthCheckerConfig {
    /// 创建新的配置
    pub fn new(check_interval: Duration, timeout: Duration) -> Self {
        Self {
            check_interval,
            timeout,
            ..Default::default()
        }
    }

    /// 设置失败阈值
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// 设置成功阈值
    pub fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }
}

/// 后台健康检查任务
pub struct BackgroundHealthChecker<H: HealthCheck> {
    /// 健康检查器
    checker: Arc<H>,
    /// 配置
    config: HealthCheckerConfig,
    /// 健康状态回调
    on_healthy: Option<Box<dyn Fn() + Send + Sync>>,
    /// 不健康状态回调
    on_unhealthy: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl<H: HealthCheck + 'static> BackgroundHealthChecker<H> {
    /// 创建新的后台健康检查器
    pub fn new(checker: Arc<H>, config: HealthCheckerConfig) -> Self {
        Self {
            checker,
            config,
            on_healthy: None,
            on_unhealthy: None,
        }
    }

    /// 设置健康状态回调
    pub fn on_healthy<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_healthy = Some(Box::new(callback));
        self
    }

    /// 设置不健康状态回调
    pub fn on_unhealthy<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_unhealthy = Some(Box::new(callback));
        self
    }

    /// 启动后台检查任务
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        let checker = self.checker;
        let config = self.config;
        let on_healthy = self.on_healthy;
        let on_unhealthy = self.on_unhealthy;

        tokio::spawn(async move {
            info!(
                component = checker.component_name(),
                interval_secs = config.check_interval.as_secs(),
                "Starting background health checker"
            );

            let mut consecutive_failures = 0u32;
            let mut consecutive_successes = 0u32;

            loop {
                tokio::time::sleep(config.check_interval).await;

                let result = tokio::time::timeout(config.timeout, checker.check()).await;

                match result {
                    Ok(check_result) => {
                        if check_result.is_healthy() {
                            consecutive_failures = 0;
                            consecutive_successes += 1;

                            if consecutive_successes >= config.success_threshold {
                                if let Some(ref callback) = on_healthy {
                                    callback();
                                }
                            }

                            debug!(
                                component = checker.component_name(),
                                latency_ms = check_result.latency_ms(),
                                "Health check passed"
                            );
                        } else {
                            consecutive_successes = 0;
                            consecutive_failures += 1;

                            let error_msg = check_result.error().unwrap_or("Unknown error");

                            if consecutive_failures >= config.failure_threshold {
                                if let Some(ref callback) = on_unhealthy {
                                    callback(error_msg);
                                }
                            }

                            error!(
                                component = checker.component_name(),
                                error = error_msg,
                                consecutive_failures,
                                "Health check failed"
                            );
                        }
                    }
                    Err(_) => {
                        consecutive_successes = 0;
                        consecutive_failures += 1;

                        if consecutive_failures >= config.failure_threshold {
                            if let Some(ref callback) = on_unhealthy {
                                callback("Health check timed out");
                            }
                        }

                        error!(
                            component = checker.component_name(),
                            consecutive_failures,
                            "Health check timed out"
                        );
                    }
                }
            }
        })
    }
}

/// 聚合多个健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedHealthResult {
    /// 整体是否健康
    pub healthy: bool,
    /// 各组件状态
    pub components: Vec<ComponentHealth>,
}

/// 组件健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// 组件名称
    pub name: String,
    /// 是否健康
    pub healthy: bool,
    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
}

impl AggregatedHealthResult {
    /// 创建新的聚合结果
    pub fn new() -> Self {
        Self {
            healthy: true,
            components: Vec::new(),
        }
    }

    /// 添加组件状态
    pub fn add_component(&mut self, name: impl Into<String>, result: BaseHealthResult) {
        let component = ComponentHealth {
            name: name.into(),
            healthy: result.healthy,
            latency_ms: result.latency_ms,
            error: result.error,
        };

        if !component.healthy {
            self.healthy = false;
        }

        self.components.push(component);
    }

    /// 获取健康组件数量
    pub fn healthy_count(&self) -> usize {
        self.components.iter().filter(|c| c.healthy).count()
    }

    /// 获取总组件数量
    pub fn total_count(&self) -> usize {
        self.components.len()
    }

    /// 是否所有组件都健康
    pub fn is_fully_healthy(&self) -> bool {
        self.healthy_count() == self.total_count()
    }

    /// 是否至少有一个组件健康
    pub fn is_partially_healthy(&self) -> bool {
        self.healthy_count() > 0
    }
}

impl Default for AggregatedHealthResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_health_result() {
        let healthy = BaseHealthResult::healthy(10);
        assert!(healthy.healthy);
        assert_eq!(healthy.latency_ms, Some(10));
        assert!(healthy.error.is_none());

        let unhealthy = BaseHealthResult::unhealthy("Connection failed");
        assert!(!unhealthy.healthy);
        assert!(unhealthy.latency_ms.is_none());
        assert_eq!(unhealthy.error, Some("Connection failed".to_string()));

        let timeout = BaseHealthResult::timeout();
        assert!(!timeout.healthy);
        assert!(timeout.error.is_some());
    }

    #[test]
    fn test_health_check_result() {
        let result: HealthCheckResult<String> =
            HealthCheckResult::healthy(15, "All good".to_string());
        assert!(result.is_healthy());
        assert_eq!(result.latency_ms(), Some(15));
        assert_eq!(result.details, Some("All good".to_string()));

        let result: HealthCheckResult<String> =
            HealthCheckResult::unhealthy("Error occurred", None);
        assert!(!result.is_healthy());
        assert_eq!(result.error(), Some("Error occurred"));
    }

    #[test]
    fn test_aggregated_health_result() {
        let mut result = AggregatedHealthResult::new();
        assert!(result.healthy);

        result.add_component("postgres", BaseHealthResult::healthy(10));
        result.add_component("redis", BaseHealthResult::healthy(5));
        assert!(result.is_fully_healthy());
        assert_eq!(result.healthy_count(), 2);

        result.add_component("kafka", BaseHealthResult::unhealthy("Connection refused"));
        assert!(!result.is_fully_healthy());
        assert!(result.is_partially_healthy());
        assert_eq!(result.healthy_count(), 2);
        assert_eq!(result.total_count(), 3);
    }

    #[test]
    fn test_health_checker_config() {
        let config = HealthCheckerConfig::new(Duration::from_secs(60), Duration::from_secs(10))
            .with_failure_threshold(5)
            .with_success_threshold(2);

        assert_eq!(config.check_interval, Duration::from_secs(60));
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 2);
    }
}
