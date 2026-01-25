//! IAM Identity Service - 身份服务入口
//!
//! 使用 cuba-bootstrap 统一启动模式

mod api;
mod application;
mod auth;
mod config;
mod domain;
mod error;
mod infrastructure;
mod oauth;
mod shared;
mod user;

use std::sync::Arc;

use auth::api::grpc::{AuthServiceImpl, AuthServiceServer};
use auth::domain::repositories::SessionRepository;
use auth::infrastructure::cache::{AuthCache, RedisAuthCache};
use auth::infrastructure::persistence::PostgresSessionRepository;
use cuba_bootstrap::{run_with_services, Infrastructure};
use cuba_ports::CachePort;
use shared::domain::repositories::UserRepository;
use shared::infrastructure::persistence::PostgresUserRepository;
use tonic::transport::Server;
use user::api::grpc::{proto::user_service_server::UserServiceServer, UserServiceImpl};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_with_services("config", |infra: Infrastructure, mut server: Server| async move {
        // 从 Infrastructure 获取资源
        let pool = infra.postgres_pool();
        let token_service = infra.token_service();
        let config = infra.config();

        // 组装 Cache（依赖 CachePort trait）
        let cache: Arc<dyn CachePort> = Arc::new(infra.redis_cache());
        let auth_cache: Arc<dyn AuthCache> = Arc::new(RedisAuthCache::new(cache));

        // 组装 Repositories（依赖 domain trait）
        let user_repo: Arc<dyn UserRepository> =
            Arc::new(PostgresUserRepository::new(pool.clone()));
        let session_repo: Arc<dyn SessionRepository> =
            Arc::new(PostgresSessionRepository::new(pool.clone()));

        // 组装 AuthService
        let auth_service = AuthServiceImpl::new(
            user_repo.clone(),
            session_repo,
            token_service.clone(),
            auth_cache,
            config.jwt.refresh_expires_in as i64,
        );

        // 组装 UserService
        let user_service = UserServiceImpl::new(user_repo, token_service);

        // 注册多个服务并启动
        let addr = format!("{}:{}", config.server.host, config.server.port)
            .parse()
            .map_err(|e| cuba_errors::AppError::internal(format!("Invalid address: {}", e)))?;

        server
            .add_service(AuthServiceServer::new(auth_service))
            .add_service(UserServiceServer::new(user_service))
            .serve_with_shutdown(addr, cuba_bootstrap::shutdown_signal())
            .await
            .map_err(|e| cuba_errors::AppError::internal(format!("Server error: {}", e)))?;

        Ok(())
    })
    .await
}
