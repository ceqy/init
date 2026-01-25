//! 服务启动器
//!
//! 提供统一的服务启动模式

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use cuba_config::AppConfig;
use cuba_errors::AppResult;
use tonic::transport::Server;
use tracing::{error, info};

use crate::health::{HealthChecker, HealthServer};
use crate::infrastructure::Infrastructure;
use crate::metrics::{MetricsRecorder, PoolMetricsCollector};
use crate::runtime::{init_runtime, shutdown_signal};

/// 服务启动器配置
pub struct ServiceConfig {
    /// 配置目录
    pub config_dir: String,
    /// 健康检查端口（默认为 gRPC 端口 + 1000）
    pub health_port: Option<u16>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            config_dir: "config".to_string(),
            health_port: None,
        }
    }
}

impl ServiceConfig {
    /// 设置健康检查端口
    pub fn with_health_port(mut self, port: u16) -> Self {
        self.health_port = Some(port);
        self
    }
}

/// 运行 gRPC 服务
///
/// 这是所有微服务的统一入口点。它负责：
/// 1. 加载配置
/// 2. 初始化运行时（日志、追踪）
/// 3. 创建基础设施资源（数据库、Redis、TokenService）
/// 4. 启动健康检查 HTTP 服务器
/// 5. 启动连接池 metrics 采集器
/// 6. 调用用户提供的闭包构建 gRPC 服务
/// 7. 启动服务器并处理 graceful shutdown
///
/// # 示例
///
/// ```ignore
/// use cuba_bootstrap::run;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     run("config", |infra| async move {
///         let service = MyServiceImpl::new(infra.postgres_pool());
///         MyServiceServer::new(service)
///     }).await
/// }
/// ```
pub async fn run<F, Fut, S>(config_dir: &str, service_builder: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(Infrastructure) -> Fut,
    Fut: Future<Output = S>,
    S: tonic::codegen::Service<
            http::Request<tonic::body::Body>,
            Response = http::Response<tonic::body::Body>,
            Error = std::convert::Infallible,
        > + tonic::server::NamedService
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: Send + 'static,
{
    // 1. 加载配置
    let config = AppConfig::load(config_dir)?;

    // 2. 初始化运行时
    init_runtime(&config);

    info!("Starting {} service", config.app_name);

    // 3. 初始化 Metrics 记录器
    let metrics = Arc::new(MetricsRecorder::new());

    // 4. 创建基础设施（带重试）
    let infra = Infrastructure::from_config(config.clone()).await?;
    let infra_arc = Arc::new(infra);

    // 5. 创建健康检查器
    let health_checker = Arc::new(HealthChecker::new());

    // 6. 启动连接池 metrics 采集器
    let pool_collector = PoolMetricsCollector::default();
    pool_collector.set_infrastructure(infra_arc.clone()).await;
    let _metrics_handle = pool_collector.start();

    // 7. 计算健康检查端口（gRPC 端口 + 1000）
    let health_port = config.server.port + 1000;

    // 8. 启动健康检查 HTTP 服务器
    let health_server = HealthServer::new(health_checker.clone(), metrics.clone(), health_port);

    // 克隆 infra 用于健康检查（需要重新创建一个 Infrastructure）
    let health_infra = Infrastructure::from_config(config.clone()).await?;
    health_server.set_infrastructure(health_infra).await;

    let health_handle = tokio::spawn(async move {
        if let Err(e) = health_server.serve().await {
            error!("Health server error: {}", e);
        }
    });

    // 9. 构建服务地址
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    // 10. 构建 gRPC 服务
    // 从 Arc 中提取 Infrastructure（需要克隆内部数据）
    let infra_for_service = Infrastructure::from_config(config.clone()).await?;
    let service = service_builder(infra_for_service).await;

    info!(%addr, "gRPC server starting");

    // 11. 启动服务器
    Server::builder()
        .add_service(service)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    // 12. 清理
    health_handle.abort();

    info!("Service stopped");

    Ok(())
}

/// 运行多个 gRPC 服务（用于需要注册多个服务的场景）
pub async fn run_with_services<F, Fut>(
    config_dir: &str,
    server_builder: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(Infrastructure, Server) -> Fut,
    Fut: Future<Output = AppResult<()>>,
{
    // 1. 加载配置
    let config = AppConfig::load(config_dir)?;

    // 2. 初始化运行时
    init_runtime(&config);

    info!("Starting {} service", config.app_name);

    // 3. 初始化 Metrics 记录器
    let metrics = Arc::new(MetricsRecorder::new());

    // 4. 创建基础设施（带重试）
    let infra = Infrastructure::from_config(config.clone()).await?;
    let infra_arc = Arc::new(infra);

    // 5. 创建健康检查器
    let health_checker = Arc::new(HealthChecker::new());

    // 6. 启动连接池 metrics 采集器
    let pool_collector = PoolMetricsCollector::default();
    pool_collector.set_infrastructure(infra_arc.clone()).await;
    let _metrics_handle = pool_collector.start();

    // 7. 计算健康检查端口（gRPC 端口 + 1000）
    let health_port = config.server.port + 1000;

    // 8. 启动健康检查 HTTP 服务器
    let health_server = HealthServer::new(health_checker.clone(), metrics.clone(), health_port);

    // 克隆 infra 用于健康检查
    let health_infra = Infrastructure::from_config(config.clone()).await?;
    health_server.set_infrastructure(health_infra).await;

    let health_handle = tokio::spawn(async move {
        if let Err(e) = health_server.serve().await {
            error!("Health server error: {}", e);
        }
    });

    // 9. 构建服务地址
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    info!(%addr, "gRPC server starting");

    // 10. 让用户构建并启动服务器
    let infra_for_service = Infrastructure::from_config(config.clone()).await?;
    let server = Server::builder();
    server_builder(infra_for_service, server).await?;

    // 11. 清理
    health_handle.abort();

    info!("Service stopped");

    Ok(())
}
