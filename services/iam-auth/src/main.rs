//! IAM Auth Service

mod api;
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;

use std::sync::Arc;

use api::grpc::{AuthServiceImpl, AuthServiceServer};
use cuba_adapter_postgres::{create_pool, PostgresConfig};
use cuba_auth_core::TokenService;
use cuba_bootstrap::init_runtime;
use cuba_config::AppConfig;
use domain::repositories::{SessionRepository, UserRepository};
use infrastructure::persistence::{PostgresSessionRepository, PostgresUserRepository};
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 加载配置
    let config = AppConfig::load("config")?;

    // 2. 初始化运行时（日志、追踪）
    init_runtime(&config);

    info!("Starting IAM Auth Service");

    // 3. 创建数据库连接池
    let pg_config = PostgresConfig::new(&config.database.url)
        .with_max_connections(config.database.max_connections);
    let pool = create_pool(&pg_config).await?;
    info!("Database connection pool created");

    // 4. 创建 TokenService
    let token_service = Arc::new(TokenService::new(
        &config.jwt.secret,
        config.jwt.expires_in as i64,
        config.jwt.refresh_expires_in as i64,
    ));

    // 5. 创建 Repositories
    let user_repo: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(pool.clone()));
    let session_repo: Arc<dyn SessionRepository> =
        Arc::new(PostgresSessionRepository::new(pool.clone()));

    // 6. 创建 AuthService
    let auth_service = AuthServiceImpl::new(
        user_repo,
        session_repo,
        token_service,
        config.jwt.refresh_expires_in as i64,
    );

    // 7. 启动 gRPC 服务器
    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    info!(%addr, "gRPC server starting");

    Server::builder()
        .add_service(AuthServiceServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
}
