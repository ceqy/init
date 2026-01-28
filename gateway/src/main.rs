//! Cuba ERP API Gateway

mod audit;
mod auth;
mod config;
mod grpc;
mod middleware;
mod routing;

use axum::{middleware as axum_middleware, Router};
use cuba_auth_core::TokenService;
use cuba_telemetry::init_tracing;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

/// 应用状态
#[derive(Clone)]
struct AppState {
    grpc_clients: grpc::GrpcClients,
    token_service: TokenService,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // 创建应用状态
    let state = AppState {
        grpc_clients,
        token_service,
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

        AppState {
            grpc_clients,
            token_service,
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
