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

use bootstrap::{Infrastructure, run_server};
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
    // 使用 bootstrap 统一启动模式
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
        use ports::EventPublisher;
        use infrastructure::events::RedisEventPublisher;
        use secrecy::ExposeSecret;

        let redis_url = config.redis.url.expose_secret();
        let event_publisher =
            RedisEventPublisher::new(redis_url, "iam_access_events").map_err(|e| {
                errors::AppError::internal(format!("Failed to connect to Redis: {}", e))
            })?;
        let event_publisher = Arc::new(event_publisher);

        // 启动 Outbox 发布器 (后台)
        let outbox_publisher = OutboxPublisher::new(outbox_repo, event_publisher.clone());
        tokio::spawn(async move {
            outbox_publisher.start().await;
        });
        info!("Outbox publisher started");

        // 初始化增强的缓存
        use infrastructure::cache::{AuthCacheConfig, CacheStrategyConfig, create_enhanced_cache};

        // 配置缓存策略
        let cache_config = CacheStrategyConfig {
            enable_multi_layer: true,          // 启用多层缓存（L1 内存 + L2 Redis）
            enable_avalanche_protection: true, // 启用雪崩防护（TTL 抖动 + Singleflight）
            enable_bloom_filter: false,        // 布隆过滤器（可选，需要 RedisBloom）
            enable_cache_warming: true,        // 启用缓存预热
            jitter_range_secs: 30,             // TTL 抖动范围：±15 秒
            auth_cache_config: AuthCacheConfig {
                user_roles_ttl_secs: 300, // 用户角色缓存 5 分钟
                role_ttl_secs: 600,       // 角色缓存 10 分钟
                policy_ttl_secs: 600,     // 策略缓存 10 分钟
            },
            ..Default::default()
        };

        // 创建增强的缓存（带雪崩防护、多层缓存、自动降级）
        let redis_conn = infra.redis_connection_manager();
        let auth_cache = create_enhanced_cache(redis_conn, cache_config.clone());

        info!("Enhanced cache initialized with multi-layer and avalanche protection");

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

        // 可选：启动缓存预热（后台任务）
        if cache_config.enable_cache_warming {
            use infrastructure::cache::start_cache_warming;

            tokio::spawn({
                let auth_cache = auth_cache.clone();
                let policy_repo = policy_repo.clone();
                let role_repo = role_repo.clone();

                async move {
                    info!("Starting cache warming...");

                    // 动态获取需要预热的租户列表
                    use crate::domain::role::RoleRepository;
                    match role_repo.list_active_tenants().await {
                        Ok(tenant_ids) => {
                            let tenant_ids: Vec<common::TenantId> = tenant_ids;
                            info!("Starting cache warming for {} tenants", tenant_ids.len());
                            if let Err(e) =
                                start_cache_warming(auth_cache, policy_repo, role_repo, tenant_ids)
                                    .await
                            {
                                tracing::warn!("Cache warming failed: {}", e);
                            } else {
                                info!("Cache warming completed successfully");
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to list active tenants for warming: {}", e);
                        }
                    }
                }
            });
        }

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
