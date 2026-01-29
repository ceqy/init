//! cuba-bootstrap - 统一服务启动骨架
//!
//! 所有服务复用的启动逻辑，包括：
//! - 配置加载
//! - 运行时初始化（日志、追踪）
//! - 基础设施资源管理（数据库、Redis、Kafka、ClickHouse、TokenService）
//! - 健康检查（/health, /ready）
//! - Metrics 导出（/metrics）
//! - 连接重试和断线重连
//! - gRPC 拦截器
//! - Graceful shutdown
//!
//! # 使用方式
//!
//! ```ignore
//! use cuba_bootstrap::{run, Infrastructure};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     run("config", |infra: Infrastructure| async move {
//!         // 从 Infrastructure 获取资源
//!         let pool = infra.postgres_pool();
//!         let cache = infra.redis_cache();
//!         let token_service = infra.token_service();
//!         let kafka = infra.kafka_producer(); // Option<Arc<KafkaEventPublisher>>
//!         let clickhouse = infra.clickhouse_client(); // Option<&ClickHouseClient>
//!
//!         // 组装服务
//!         let service = MyServiceImpl::new(pool, cache, token_service);
//!         MyServiceServer::new(service)
//!     }).await
//! }
//! ```
//!
//! # 健康检查端点
//!
//! 服务启动后会在 gRPC 端口 + 1000 上提供 HTTP 健康检查端点：
//! - `GET /health` - 存活检查（liveness），始终返回 200
//! - `GET /ready` - 就绪检查（readiness），检查所有依赖
//! - `GET /metrics` - Prometheus 格式的 metrics

mod health;
mod infrastructure;
mod interceptor;
mod metrics;
mod retry;
mod runtime;
mod shutdown;
mod starter;

pub use health::*;
pub use infrastructure::{Infrastructure, PoolStatus};
pub use interceptor::*;
pub use metrics::*;
pub use retry::*;
pub use runtime::*;
pub use shutdown::*;
pub use starter::{ServiceConfig, run, run_server, run_with_services};
