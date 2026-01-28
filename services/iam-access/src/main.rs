#![allow(dead_code)]
#![allow(unused_imports)]

//! IAM Access Service - 访问控制服务入口
//!
//! 负责 RBAC, Policy 管理和统一鉴权

mod api;
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;

use std::sync::Arc;

use cuba_bootstrap::{Infrastructure, run_server};
use tonic_reflection::server::Builder as ReflectionBuilder;
use tracing::info;

use api::proto::authorization::authorization_service_server::AuthorizationServiceServer;
use api::proto::policy::policy_service_server::PolicyServiceServer;
use api::proto::rbac::rbac_service_server::RbacServiceServer;
use api::{AuthorizationServiceImpl, PolicyServiceImpl, RbacServiceImpl};
use application::policy::PolicyCommandHandler;
use infrastructure::persistence::{
    OutboxPublisher, PostgresOutboxRepository, PostgresPermissionRepository,
    PostgresPolicyRepository, PostgresRoleRepository, PostgresUnitOfWorkFactory,
    PostgresUserRoleRepository,
};

/// 文件描述符集 (用于 gRPC 反射)
pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("iam_access_descriptor");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 cuba-bootstrap 统一启动模式
    run_server("config", |infra: Infrastructure, mut server| async move {
        info!("Initializing IAM Access Service...");

        // 获取基础设施资源
        let pool = infra.postgres_pool();
        let config = infra.config();

        // 初始化仓储
        let permission_repo = Arc::new(PostgresPermissionRepository::new(pool.clone()));
        let role_repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
        let policy_repo = Arc::new(PostgresPolicyRepository::new(pool.clone()));
        // user_role_repo initialized later with cache
        let outbox_repo = Arc::new(PostgresOutboxRepository::new(pool.clone()));

        info!("Repositories initialized");

        // 初始化 Unit of Work
        let uow_factory = Arc::new(PostgresUnitOfWorkFactory::new(pool.clone()));

        // 初始化 EventPublisher (Redis)
        use cuba_ports::EventPublisher;
        use infrastructure::events::RedisEventPublisher;
        use secrecy::ExposeSecret;

        let redis_url = config.redis.url.expose_secret();
        let event_publisher =
            RedisEventPublisher::new(redis_url, "iam_access_events").map_err(|e| {
                cuba_errors::AppError::internal(format!("Failed to connect to Redis: {}", e))
            })?;
        let event_publisher = Arc::new(event_publisher);

        // 启动 Outbox 发布器 (后台)
        let outbox_publisher = OutboxPublisher::new(outbox_repo, event_publisher.clone());
        tokio::spawn(async move {
            outbox_publisher.start().await;
        });
        info!("Outbox publisher started");

        // 初始化缓存
        use infrastructure::cache::AuthCache;
        let redis_cache = infra.redis_cache();
        let auth_cache = Arc::new(AuthCache::new(Arc::new(redis_cache)));

        // 更新 UserRoleRepository 使用缓存
        let user_role_repo =
            Arc::new(PostgresUserRoleRepository::new(pool.clone()).with_cache(auth_cache.clone()));

        // 创建 gRPC 服务
        let rbac_service = RbacServiceImpl::new(
            role_repo.clone(),
            permission_repo.clone(),
            user_role_repo.clone(),
            uow_factory.clone(),
        );

        let policy_cmd_handler = PolicyCommandHandler::new(uow_factory.clone());
        let policy_service = PolicyServiceImpl::new(policy_cmd_handler, policy_repo.clone());

        let authorization_service =
            AuthorizationServiceImpl::new(policy_repo.clone(), user_role_repo.clone());

        info!("gRPC services created");

        // 构建反射服务
        let reflection_service = ReflectionBuilder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .unwrap();

        // 注册服务并启动
        // 使用 with_interceptor 注入追踪拦截器
        use api::grpc::tracing_interceptor;

        Ok(server
            .add_service(RbacServiceServer::with_interceptor(
                rbac_service,
                tracing_interceptor,
            ))
            .add_service(AuthorizationServiceServer::with_interceptor(
                authorization_service,
                tracing_interceptor,
            ))
            .add_service(PolicyServiceServer::with_interceptor(
                policy_service,
                tracing_interceptor,
            ))
            .add_service(reflection_service))
    })
    .await
}
