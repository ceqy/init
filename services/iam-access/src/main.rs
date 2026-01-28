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

use cuba_bootstrap::{run_server, Infrastructure};
use tonic_reflection::server::Builder as ReflectionBuilder;
use tracing::info;

use api::proto::rbac::rbac_service_server::RbacServiceServer;
use api::proto::authorization::authorization_service_server::AuthorizationServiceServer;
use api::{RbacServiceImpl, AuthorizationServiceImpl};
use infrastructure::{
    PostgresPermissionRepository,
    PostgresPolicyRepository,
    PostgresRoleRepository,
    PostgresRolePermissionRepository,
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
        let role_permission_repo = Arc::new(PostgresRolePermissionRepository::new(pool.clone()));

        info!("Repositories initialized");

        // 初始化 EventPublisher (Redis)
        use infrastructure::events::RedisEventPublisher;
        use cuba_ports::EventPublisher;
        use secrecy::ExposeSecret;

        let redis_url = config.redis.url.expose_secret();
        let event_publisher = RedisEventPublisher::new(redis_url, "iam_access_events")
             .map_err(|e| cuba_errors::AppError::internal(format!("Failed to connect to Redis: {}", e)))?;
        let event_publisher = Arc::new(event_publisher);

        // 初始化缓存
        use infrastructure::cache::AuthCache;
        let redis_cache = infra.redis_cache();
        let auth_cache = Arc::new(AuthCache::new(Arc::new(redis_cache)));

        // 更新 UserRoleRepository 使用缓存
        let user_role_repo = Arc::new(
            PostgresUserRoleRepository::new(pool.clone())
                .with_cache(auth_cache.clone())
        );

        // 创建 gRPC 服务
        let rbac_service = RbacServiceImpl::new(
            role_repo.clone(),
            permission_repo.clone(),
            user_role_repo.clone(),
            role_permission_repo.clone(),
            event_publisher.clone(),
        );

        let authorization_service = AuthorizationServiceImpl::new(
            policy_repo.clone(),
            user_role_repo.clone(),
        );

        info!("gRPC services created");

        // 构建反射服务
        let reflection_service = ReflectionBuilder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .unwrap();

        // 注册服务并启动
        Ok(server
            .add_service(RbacServiceServer::new(rbac_service))
            .add_service(AuthorizationServiceServer::new(authorization_service))
            .add_service(reflection_service))
    })
    .await
}
