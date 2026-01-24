//! Cuba ERP API Gateway

mod auth;
mod config;
mod middleware;
mod routing;

use axum::Router;
use cuba_telemetry::init_tracing;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    // 初始化 tracing
    init_tracing("info");

    // 加载配置
    let config = config::GatewayConfig::from_env();

    // 构建路由
    let app = Router::new()
        .merge(routing::api_routes())
        .merge(auth::auth_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // 启动服务器
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid address");

    info!(%addr, "Starting gateway");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
