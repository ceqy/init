//! Metrics 模块
//!
//! 提供 Prometheus metrics 导出

use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

use crate::infrastructure::{Infrastructure, PoolStatus};

/// Metrics 记录器
pub struct MetricsRecorder {
    handle: PrometheusHandle,
}

impl MetricsRecorder {
    /// 创建新的 Metrics 记录器
    pub fn new() -> Self {
        let handle = PrometheusBuilder::new()
            .install_recorder()
            .expect("Failed to install Prometheus recorder");

        Self { handle }
    }

    /// 获取 Prometheus 格式的 metrics
    pub fn render(&self) -> String {
        self.handle.render()
    }
}

impl Default for MetricsRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// 记录 gRPC 请求
pub fn record_grpc_request(service: &str, method: &str, status: &str, duration_ms: f64) {
    let labels = [
        ("service", service.to_string()),
        ("method", method.to_string()),
        ("status", status.to_string()),
    ];

    counter!("grpc_requests_total", &labels).increment(1);
    histogram!("grpc_request_duration_ms", &labels).record(duration_ms);
}

/// 记录数据库查询
pub fn record_db_query(operation: &str, table: &str, duration_ms: f64, success: bool) {
    let labels = [
        ("operation", operation.to_string()),
        ("table", table.to_string()),
        ("success", success.to_string()),
    ];

    counter!("db_queries_total", &labels).increment(1);
    histogram!("db_query_duration_ms", &labels).record(duration_ms);
}

/// 记录缓存操作
pub fn record_cache_operation(operation: &str, hit: bool) {
    let labels = [
        ("operation", operation.to_string()),
        ("hit", hit.to_string()),
    ];

    counter!("cache_operations_total", &labels).increment(1);
}

/// 记录 Kafka 消息
pub fn record_kafka_message(topic: &str, operation: &str, success: bool) {
    let labels = [
        ("topic", topic.to_string()),
        ("operation", operation.to_string()),
        ("success", success.to_string()),
    ];

    counter!("kafka_messages_total", &labels).increment(1);
}

/// 设置连接池大小
pub fn set_pool_size(pool_name: &str, size: usize) {
    let labels = [("pool", pool_name.to_string())];
    gauge!("connection_pool_size", &labels).set(size as f64);
}

/// 设置活跃连接数
pub fn set_active_connections(pool_name: &str, count: usize) {
    let labels = [("pool", pool_name.to_string())];
    gauge!("connection_pool_active", &labels).set(count as f64);
}

/// 记录连接获取等待时间
pub fn record_connection_acquire_duration(pool_name: &str, duration_ms: f64, success: bool) {
    let labels = [
        ("pool", pool_name.to_string()),
        ("success", success.to_string()),
    ];
    histogram!("connection_acquire_duration_ms", &labels).record(duration_ms);
}

/// 记录连接获取失败
pub fn record_connection_acquire_failure(pool_name: &str, reason: &str) {
    let labels = [
        ("pool", pool_name.to_string()),
        ("reason", reason.to_string()),
    ];
    counter!("connection_acquire_failures_total", &labels).increment(1);
}

/// 记录连接超时
pub fn record_connection_timeout(pool_name: &str) {
    let labels = [("pool", pool_name.to_string())];
    counter!("connection_timeouts_total", &labels).increment(1);
}

/// 设置连接池使用率
pub fn set_pool_utilization(pool_name: &str, utilization: f64) {
    let labels = [("pool", pool_name.to_string())];
    gauge!("connection_pool_utilization", &labels).set(utilization);
}

/// 记录事件发布
pub fn record_event_publish(event_type: &str, publisher: &str, success: bool, duration_ms: f64) {
    let labels = [
        ("event_type", event_type.to_string()),
        ("publisher", publisher.to_string()),
        ("success", success.to_string()),
    ];
    counter!("event_publish_total", &labels).increment(1);
    histogram!("event_publish_duration_ms", &labels).record(duration_ms);
}

/// 记录 Outbox 处理
pub fn record_outbox_processing(batch_size: usize, processed: usize, failed: usize) {
    gauge!("outbox_batch_size").set(batch_size as f64);
    counter!("outbox_messages_processed_total").increment(processed as u64);
    counter!("outbox_messages_failed_total").increment(failed as u64);
}

/// 请求计时器
pub struct RequestTimer {
    start: Instant,
    service: String,
    method: String,
}

impl RequestTimer {
    pub fn new(service: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            service: service.into(),
            method: method.into(),
        }
    }

    pub fn finish(self, status: &str) {
        let duration = self.start.elapsed().as_secs_f64() * 1000.0;
        record_grpc_request(&self.service, &self.method, status, duration);
    }
}

/// 数据库查询计时器
pub struct DbQueryTimer {
    start: Instant,
    operation: String,
    table: String,
}

impl DbQueryTimer {
    pub fn new(operation: impl Into<String>, table: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.into(),
            table: table.into(),
        }
    }

    pub fn finish(self, success: bool) {
        let duration = self.start.elapsed().as_secs_f64() * 1000.0;
        record_db_query(&self.operation, &self.table, duration, success);
    }
}

/// 连接池 Metrics 采集器
///
/// 定期采集 PostgreSQL 和 Redis 连接池状态
pub struct PoolMetricsCollector {
    infra: Arc<RwLock<Option<Arc<Infrastructure>>>>,
    interval: Duration,
}

impl PoolMetricsCollector {
    /// 创建新的连接池 Metrics 采集器
    pub fn new(interval: Duration) -> Self {
        Self {
            infra: Arc::new(RwLock::new(None)),
            interval,
        }
    }

    /// 设置基础设施引用
    pub async fn set_infrastructure(&self, infra: Arc<Infrastructure>) {
        let mut guard = self.infra.write().await;
        *guard = Some(infra);
    }

    /// 启动后台采集任务
    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let infra = self.infra.clone();
        let interval = self.interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;

                let guard = infra.read().await;
                if let Some(infra) = guard.as_ref() {
                    // 采集 PostgreSQL 连接池指标
                    let pool_status = infra.postgres_pool_status();
                    record_postgres_pool_metrics(&pool_status);

                    // 采集 Redis 连接状态
                    let redis_connected = infra.check_redis_connection().await;
                    record_redis_connection_status(redis_connected);

                    debug!(
                        postgres_write_size = pool_status.write_size,
                        postgres_write_idle = pool_status.write_idle,
                        postgres_write_active = pool_status.write_active,
                        postgres_read_size = pool_status.read_size,
                        postgres_read_idle = pool_status.read_idle,
                        postgres_read_active = pool_status.read_active,
                        redis_connected = redis_connected,
                        "Pool metrics collected"
                    );
                }
            }
        })
    }
}

impl Default for PoolMetricsCollector {
    fn default() -> Self {
        Self::new(Duration::from_secs(15))
    }
}

/// 记录 PostgreSQL 连接池指标
pub fn record_postgres_pool_metrics(status: &PoolStatus) {
    // 记录写连接池指标
    let write_labels = [("pool", "write".to_string())];
    gauge!("postgres_pool_size", &write_labels).set(status.write_size as f64);
    gauge!("postgres_pool_idle", &write_labels).set(status.write_idle as f64);
    gauge!("postgres_pool_active", &write_labels).set(status.write_active as f64);

    // 计算写连接池使用率
    let write_utilization = if status.write_size > 0 {
        (status.write_active as f64 / status.write_size as f64) * 100.0
    } else {
        0.0
    };
    gauge!("postgres_pool_utilization", &write_labels).set(write_utilization);

    // 如果有读连接池，记录读连接池指标
    if status.read_size > 0 {
        let read_labels = [("pool", "read".to_string())];
        gauge!("postgres_pool_size", &read_labels).set(status.read_size as f64);
        gauge!("postgres_pool_idle", &read_labels).set(status.read_idle as f64);
        gauge!("postgres_pool_active", &read_labels).set(status.read_active as f64);

        // 计算读连接池使用率
        let read_utilization = if status.read_size > 0 {
            (status.read_active as f64 / status.read_size as f64) * 100.0
        } else {
            0.0
        };
        gauge!("postgres_pool_utilization", &read_labels).set(read_utilization);
    }

    // 记录总体使用率（用于向后兼容）
    let total_size = status.write_size + status.read_size;
    let total_active = status.write_active + status.read_active;
    let total_utilization = if total_size > 0 {
        (total_active as f64 / total_size as f64) * 100.0
    } else {
        write_utilization
    };
    set_pool_utilization("postgres", total_utilization);
}

/// 记录 Redis 连接状态
pub fn record_redis_connection_status(connected: bool) {
    gauge!("redis_connection_status").set(if connected { 1.0 } else { 0.0 });
}
