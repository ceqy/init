//! Cuba ERP API Gateway

mod audit;
mod auth;
mod config;
mod grpc;
mod middleware;
mod rate_limit;
mod routing;
mod security_headers;
mod ws;

use std::sync::Arc;
use tokio::sync::broadcast;

use axum::{http::HeaderValue, middleware as axum_middleware, Router};
use cuba_auth_core::TokenService;
use cuba_telemetry::init_tracing;
use std::net::SocketAddr;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use futures::StreamExt;

/// 应用状态
#[derive(Clone)]
struct AppState {
    grpc_clients: grpc::GrpcClients,
    token_service: TokenService,
    notify_tx: Arc<broadcast::Sender<String>>,
    rate_limit_config: Arc<rate_limit::RateLimitConfig>,
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
    // 注意：issuer 必须与 IAM 服务 (bootstrap/infrastructure.rs) 中的配置一致
    let token_service = TokenService::new(
        &config.jwt_secret,
        3600,  // access_token_expires_in: 1 小时
        86400 * 7,  // refresh_token_expires_in: 7 天
        "cuba-iam".to_string(),  // issuer - 必须与 IAM 服务一致
        "cuba-api".to_string(),  // audience
    );

    // 初始化 gRPC 客户端
    info!("Connecting to IAM service at {}", config.iam_endpoint);
    let grpc_clients = grpc::GrpcClients::new(config.iam_endpoint.clone())
        .await
        .expect("Failed to connect to IAM service");
    
    // 创建限流配置
    let rate_limit_config = Arc::new(rate_limit::RateLimitConfig::new(
        config.redis_url.clone(),
        60,   // 时间窗口：60 秒
        100,  // 最大请求数：100 次/分钟
    ));
    
    // 创建广播通道 (容量 100)
    let (notify_tx, _rx) = broadcast::channel::<String>(100);
    let notify_tx = Arc::new(notify_tx);
    let notify_tx_clone = notify_tx.clone();

    // 启动 Redis 订阅任务
    let redis_url = config.redis_url.clone();
    tokio::spawn(async move {
        // 连接 Redis
        let client = match redis::Client::open(redis_url) {
            Ok(c) => c,
            Err(e) => {
                info!("Failed to create Redis client for pubsub: {}", e);
                return;
            }
        };
        
        // 订阅 - MultiplexedConnection 不支持 pubsub，需要使用普通连接
        // Note: get_async_connection() is deprecated but required for pubsub functionality
        // The recommended get_multiplexed_async_connection() doesn't support into_pubsub()
        #[allow(deprecated)]
        let con = match client.get_async_connection().await {
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
        rate_limit_config,
    };


    let app = create_app(state, &config);

    // 启动服务器
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid address");

    info!(%addr, "Starting gateway");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_app(state: AppState, config: &config::GatewayConfig) -> Router {
    // 构建路由
    // 无状态的路由（健康检查等）
    let stateless_routes = routing::api_routes();

    // 公共路由（不需要认证，但需要限流保护）
    let public_routes = auth::auth_routes()
        .layer(axum_middleware::from_fn_with_state(
            state.rate_limit_config.clone(),
            rate_limit::rate_limit_middleware,
        ))
        .with_state(state.grpc_clients.clone());

    // 受保护的路由（需要认证）
    let ws_state = ws::WsState {
        notify_tx: state.notify_tx.clone(),
        token_service: state.token_service.clone(),
    };

    let protected_routes = Router::new()
        .route("/api/auth/me", axum::routing::get(auth::get_current_user))
        .nest("/api/audit", audit::audit_routes())
        .route("/ws/events", axum::routing::get(ws::websocket_handler).with_state(ws_state))
        .layer(axum_middleware::from_fn_with_state(
            state.token_service.clone(),
            middleware::auth_middleware,
        ));

    // 配置 CORS
    let cors = if config.cors_allowed_origins.is_empty() {
        // 开发模式：允许所有来源
        info!("CORS: Permissive mode (allowing all origins)");
        CorsLayer::permissive()
    } else {
        // 生产模式：只允许配置的来源
        info!("CORS: Restricted mode, allowed origins: {:?}", config.cors_allowed_origins);
        let origins: Vec<HeaderValue> = config
            .cors_allowed_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();
        
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::PATCH,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
                axum::http::header::ACCEPT,
            ])
            .allow_credentials(true)
    };

    // 合并所有路由：先合并带状态的路由，再合并无状态路由，最后应用中间件
    public_routes
        .merge(protected_routes.with_state(state.grpc_clients))
        .merge(stateless_routes)
        .layer(axum_middleware::from_fn(security_headers::security_headers_middleware))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))  // 10 MB 请求体限制
        .layer(TraceLayer::new_for_http())
        .layer(cors)
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
    use crate::grpc::{
        auth::auth_service_client::AuthServiceClient,
        user::user_service_client::UserServiceClient,
        audit::audit_service_client::AuditServiceClient,
    };

    fn create_test_state() -> AppState {
        let token_service = TokenService::new(
            "test_secret_at_least_32_characters_long",
            3600,
            3600,
            "test-issuer".to_string(),
            "test-audience".to_string(),
        );
        // Create a lazy channel that doesn't strictly connect immediately
        let channel = Channel::from_static("http://[::1]:50051").connect_lazy();
        
        let grpc_clients = grpc::GrpcClients {
            auth: AuthServiceClient::new(channel.clone()),
            user: UserServiceClient::new(channel.clone()),
            audit: AuditServiceClient::new(channel),
        };

        // Mock broadcast channel for tests
        let (notify_tx, _rx) = broadcast::channel::<String>(100);
        let notify_tx = Arc::new(notify_tx);

        // Mock rate limit config for tests
        let rate_limit_config = Arc::new(rate_limit::RateLimitConfig::new(
            "redis://localhost:6379".to_string(),
            60,
            100,
        ));

        AppState {
            grpc_clients,
            token_service,
            notify_tx,
            rate_limit_config,
        }
    }

    fn create_test_config() -> config::GatewayConfig {
        config::GatewayConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            jwt_secret: "test_secret_at_least_32_characters_long".to_string(),
            iam_endpoint: "http://127.0.0.1:50051".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            cors_allowed_origins: vec!["http://localhost:3000".to_string()],
        }
    }

    #[tokio::test]
    async fn test_health_check_public() {
        let state = create_test_state();
        let config = create_test_config();
        let app = create_app(state, &config);

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
        let config = create_test_config();
        let app = create_app(state, &config);

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
        let config = create_test_config();
        let app = create_app(state, &config);

        // Access protected route without token
        let req = Request::builder()
            .uri("/api/auth/me")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
