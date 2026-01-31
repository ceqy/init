//! telemetry - 可观测性库

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// 初始化 tracing
pub fn init_tracing(log_level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// 初始化 JSON 格式的 tracing（生产环境）
pub fn init_tracing_json(log_level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

/// 初始化 Prometheus metrics
pub fn init_metrics() -> metrics_exporter_prometheus::PrometheusHandle {
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    builder
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}

/// 健康检查状态
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub checks: Vec<HealthCheck>,
}

#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub healthy: bool,
    pub message: Option<String>,
}

impl HealthStatus {
    pub fn new() -> Self {
        Self {
            healthy: true,
            checks: Vec::new(),
        }
    }

    pub fn add_check(&mut self, name: impl Into<String>, healthy: bool, message: Option<String>) {
        if !healthy {
            self.healthy = false;
        }
        self.checks.push(HealthCheck {
            name: name.into(),
            healthy,
            message,
        });
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::new()
    }
}
