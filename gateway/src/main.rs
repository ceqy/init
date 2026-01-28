//! Cuba ERP API Gateway

mod audit;
mod auth;
mod config;
mod grpc;
mod middleware;
mod routing;
mod ws;

use std::sync::Arc;
use tokio::sync::broadcast;
use redis::AsyncCommands;
use secrecy::ExposeSecret;

use axum::{middleware as axum_middleware, Router};
use cuba_auth_core::TokenService;
use cuba_telemetry::init_tracing;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use futures::StreamExt;

/// 应用状态
#[derive(Clone)]
struct AppState {
    grpc_clients: grpc::GrpcClients,
    token_service: TokenService,
    notify_tx: Arc<broadcast::Sender<String>>,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 .env 文件
    dotenvy::dotenv().ok();

    // 初始化 tracing
    init_tracing("info");

    // 加载配置
    let config = config::GatewayConfig::from_env();

    // 初始化 TokenService
    let token_service = TokenService::new(
        &config.jwt_secret,
        3600,  // access_token_expires_in: 1 小时
        86400 * 7,  // refresh_token_expires_in: 7 天
    );

    // 初始化 gRPC 客户端
    info!("Connecting to IAM service at {}", config.iam_endpoint);
    let grpc_clients = grpc::GrpcClients::new(config.iam_endpoint.clone())
        .await
        .expect("Failed to connect to IAM service");
    // 创建广播通道 (容量 100)
    let (notify_tx, _rx) = broadcast::channel::<String>(100);
    let notify_tx = Arc::new(notify_tx);
    let notify_tx_clone = notify_tx.clone();

    // 启动 Redis 订阅任务
    let redis_url = config.redis_url.clone(); 
    // 注意：GatewayConfig 可能还没暴露 redis_url，假设它有。如果没有，需要先去 config.rs 添加。
    // 假设 config.rs 还没有 redis_url，我们这里先用硬编码或者稍后修改 config.rs。
    // 为了稳妥，先假设需要修改 config.rs。此处先留个 TODO 或者直接假装 config 有。
    // 实际上我们在上一步并没有修改 GatewayConfig，所以这里肯定会报错。
    // 我们先写好逻辑，下一步修 config。
    tokio::spawn(async move {
        // 连接 Redis
        let client = match redis::Client::open(redis_url) {
            Ok(c) => c,
            Err(e) => {
                info!("Failed to create Redis client for pubsub: {}", e);
                return;
            }
        };
        
        // 订阅
        let mut con = match client.get_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                info!("Failed to connect to Redis for pubsub: {}", e);
                return;
            }
        };
        
        let mut pubsub = con.into_pubsub();
        if let Err(e) = pubsub.subscribe("domain_events").await {
             info!("Failed to subscribe to domain_events: {}", e);
             return;
        }

        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
             let payload: String = match msg.get_payload() {
                 Ok(p) => p,
                 Err(_) => continue,
             };
             // 广播给所有 WebSocket 客户端
             let _ = notify_tx_clone.send(payload);
        }
    });

    // 创建应用状态
    let state = AppState {
        grpc_clients,
        token_service,
        notify_tx,
    };


    let app = create_app(state);

    // 启动服务器
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid address");

    info!(%addr, "Starting gateway");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_app(state: AppState) -> Router {
    // 构建路由
    // 无状态的路由（健康检查等）
    let stateless_routes = routing::api_routes();

    // 公共路由（不需要认证）
    let public_routes = auth::auth_routes().with_state(state.grpc_clients.clone());

    // 受保护的路由（需要认证）
    let protected_routes = Router::new()
        .route("/api/auth/me", axum::routing::get(auth::get_current_user))
        .nest("/api/audit", audit::audit_routes())
        .route("/ws/events", axum::routing::get(ws::websocket_handler).with_state(state.notify_tx))
        .route_layer(axum_middleware::from_fn_with_state(
            state.token_service.clone(),
            middleware::auth_middleware,
        ));

    // 合并所有路由：先合并带状态的路由，再合并无状态路由，最后应用中间件
    public_routes
        .merge(protected_routes.with_state(state.grpc_clients))
        .merge(stateless_routes)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use tonic::transport::Channel;
    use crate::grpc::{auth::auth_service_client::AuthServiceClient, user::user_service_client::UserServiceClient};

    fn create_test_state() -> AppState {
        let token_service = TokenService::new("test_secret", 3600, 3600);
        // Create a lazy channel that doesn't strictly connect immediately
        let channel = Channel::from_static("http://[::1]:50051").connect_lazy();
        
        let grpc_clients = grpc::GrpcClients {
            auth: AuthServiceClient::new(channel.clone()),
            user: UserServiceClient::new(channel),
        };

        // Mock broadcast channel for tests
        let (notify_tx, _rx) = broadcast::channel::<String>(100);
        let notify_tx = Arc::new(notify_tx);

        AppState {
            grpc_clients,
            token_service,
            notify_tx,
        }
    }

    #[tokio::test]
    async fn test_health_check_public() {
        let state = create_test_state();
        let app = create_app(state);

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_login_public_no_auth_required() {
        let state = create_test_state();
        let app = create_app(state);

        // POST to login, should reach handler, not be blocked by 401
        // Since we send empty body, likely 400 or 422 or 500, but NOT 401
        let req = Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_current_user_protected_no_token() {
        let state = create_test_state();
        let app = create_app(state);

        // Access protected route without token
        let req = Request::builder()
            .uri("/api/auth/me")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
