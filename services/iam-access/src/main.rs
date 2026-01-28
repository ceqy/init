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

use cuba_bootstrap::{run_with_services, Infrastructure};
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
    run_with_services("config", |infra: Infrastructure, mut server| async move {
        info!("Initializing IAM Access Service...");

        // 获取基础设施资源
        let pool = infra.postgres_pool();
        let config = infra.config();

        // 初始化仓储
        let permission_repo = Arc::new(PostgresPermissionRepository::new(pool.clone()));
        let role_repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
        let policy_repo = Arc::new(PostgresPolicyRepository::new(pool.clone()));
        let user_role_repo = Arc::new(PostgresUserRoleRepository::new(pool.clone()));
        let role_permission_repo = Arc::new(PostgresRolePermissionRepository::new(pool.clone()));

        info!("Repositories initialized");

        // 创建 gRPC 服务
        let rbac_service = RbacServiceImpl::new(
            role_repo.clone(),
            permission_repo.clone(),
            user_role_repo.clone(),
            role_permission_repo.clone(),
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
            .map_err(|e| cuba_errors::AppError::internal(format!("Failed to build reflection service: {}", e)))?;

        // 解析服务地址
        let addr = format!("{}:{}", config.server.host, config.server.port)
            .parse()
            .map_err(|e| cuba_errors::AppError::internal(format!("Invalid address: {}", e)))?;

        info!("IAM Access Service starting on {}", addr);

        // 注册服务并启动
        server
            .add_service(RbacServiceServer::new(rbac_service))
            .add_service(AuthorizationServiceServer::new(authorization_service))
            .add_service(reflection_service)
            .serve_with_shutdown(addr, cuba_bootstrap::shutdown_signal())
            .await
            .map_err(|e| cuba_errors::AppError::internal(format!("Server error: {}", e)))?;

        Ok(())
    })
    .await
}
