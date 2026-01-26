//! Cuba ERP API Gateway

mod auth;
mod config;
mod grpc;
mod middleware;
mod routing;

use axum::Router;
use cuba_telemetry::init_tracing;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 tracing
    init_tracing("info");

    // 加载配置
    let config = config::GatewayConfig::from_env();

    // 初始化 gRPC 客户端
    info!("Connecting to IAM service at {}", config.iam_endpoint);
    let grpc_clients = grpc::GrpcClients::new(config.iam_endpoint.clone())
        .await
        .expect("Failed to connect to IAM service");

    // 构建路由（先创建带状态的路由，再合并无状态的路由）
    let app = auth::auth_routes()
        .with_state(grpc_clients)
        .merge(routing::api_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // 启动服务器
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid address");

    info!(%addr, "Starting gateway");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
