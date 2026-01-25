//! 服务启动器
//!
//! 提供统一的服务启动模式

use std::future::Future;
use std::net::SocketAddr;

use cuba_config::AppConfig;
use cuba_errors::AppResult;
use tonic::transport::Server;
use tracing::info;

use crate::infrastructure::Infrastructure;
use crate::runtime::{init_runtime, shutdown_signal};

/// 服务启动器配置
pub struct ServiceConfig {
    /// 配置目录
    pub config_dir: String,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            config_dir: "config".to_string(),
        }
    }
}

/// 运行 gRPC 服务
///
/// 这是所有微服务的统一入口点。它负责：
/// 1. 加载配置
/// 2. 初始化运行时（日志、追踪）
/// 3. 创建基础设施资源（数据库、Redis、TokenService）
/// 4. 调用用户提供的闭包构建 gRPC 服务
/// 5. 启动服务器并处理 graceful shutdown
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

    // 3. 创建基础设施
    let infra = Infrastructure::from_config(config.clone()).await?;

    // 4. 构建服务地址
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    // 5. 构建 gRPC 服务
    let service = service_builder(infra).await;

    info!(%addr, "gRPC server starting");

    // 6. 启动服务器
    Server::builder()
        .add_service(service)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

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

    // 3. 创建基础设施
    let infra = Infrastructure::from_config(config.clone()).await?;

    // 4. 构建服务地址
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    info!(%addr, "gRPC server starting");

    // 5. 让用户构建并启动服务器
    let server = Server::builder();
    server_builder(infra, server).await?;

    info!("Service stopped");

    Ok(())
}
