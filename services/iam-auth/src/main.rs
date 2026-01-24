//! IAM Auth Service

mod api;
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;

use api::grpc::{create_auth_service, AuthServiceServer};
use cuba_bootstrap::init_runtime;
use cuba_config::AppConfig;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let config = AppConfig::load("config")?;

    // 初始化运行时
    init_runtime(&config);

    info!("Starting IAM Auth Service");

    // 创建服务
    let auth_service = create_auth_service().await?;

    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    info!(%addr, "gRPC server starting");

    // 启动 gRPC 服务器
    Server::builder()
        .add_service(AuthServiceServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
}
