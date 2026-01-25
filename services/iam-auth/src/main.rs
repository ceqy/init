//! IAM Auth Service - 认证服务入口
//!
//! 使用 cuba-bootstrap 统一启动模式

mod api;
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;

use std::sync::Arc;

use api::grpc::{AuthServiceImpl, AuthServiceServer};
use cuba_bootstrap::{run, Infrastructure};
use cuba_ports::CachePort;
use domain::repositories::{SessionRepository, UserRepository};
use infrastructure::cache::{AuthCache, RedisAuthCache};
use infrastructure::persistence::{PostgresSessionRepository, PostgresUserRepository};

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
