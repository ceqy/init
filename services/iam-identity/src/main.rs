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
use cuba_bootstrap::{run, Infrastructure};
use cuba_ports::CachePort;
use shared::domain::repositories::UserRepository;
use shared::infrastructure::persistence::PostgresUserRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run("config", |infra: Infrastructure| async move {
        // 从 Infrastructure 获取资源
        let pool = infra.postgres_pool();
        let token_service = infra.token_service();
        let config = infra.config();

        // 组装 Cache（依赖 CachePort trait）
        let cache: Arc<dyn CachePort> = Arc::new(infra.redis_cache());
        let auth_cache: Arc<dyn AuthCache> = Arc::new(RedisAuthCache::new(cache));

        // 组装 Repositories（依赖 domain trait）
        let user_repo: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(pool.clone()));
        let session_repo: Arc<dyn SessionRepository> =
            Arc::new(PostgresSessionRepository::new(pool.clone()));

        // 组装 AuthService
        let auth_service = AuthServiceImpl::new(
            user_repo,
            session_repo,
            token_service,
            auth_cache,
            config.jwt.refresh_expires_in as i64,
        );

        AuthServiceServer::new(auth_service)
    })
    .await
}
