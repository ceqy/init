//! 基础设施资源管理
//!
//! 统一管理所有微服务共享的基础设施资源

use std::sync::Arc;

use cuba_adapter_postgres::{create_pool, PostgresConfig};
use cuba_adapter_redis::{create_connection_manager, RedisCache};
use cuba_auth_core::TokenService;
use cuba_config::AppConfig;
use cuba_errors::AppResult;
use cuba_ports::CachePort;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use tracing::info;

/// 基础设施资源容器
///
/// 包含所有微服务共享的基础设施资源，由 bootstrap 统一初始化
pub struct Infrastructure {
    /// 应用配置
    config: AppConfig,
    /// PostgreSQL 连接池
    postgres_pool: PgPool,
    /// Redis 连接管理器
    redis_conn: ConnectionManager,
    /// Token 服务
    token_service: Arc<TokenService>,
}

impl Infrastructure {
    /// 从配置创建基础设施资源
    pub async fn from_config(config: AppConfig) -> AppResult<Self> {
        // 1. 创建 PostgreSQL 连接池
        let pg_config = PostgresConfig::new(&config.database.url)
            .with_max_connections(config.database.max_connections);
        let postgres_pool = create_pool(&pg_config).await?;
        info!("PostgreSQL connection pool created");

        // 2. 创建 Redis 连接
        let redis_conn = create_connection_manager(&config.redis.url).await?;
        info!("Redis connection created");

        // 3. 创建 TokenService
        let token_service = Arc::new(TokenService::new(
            &config.jwt.secret,
            config.jwt.expires_in as i64,
            config.jwt.refresh_expires_in as i64,
        ));

        Ok(Self {
            config,
            postgres_pool,
            redis_conn,
            token_service,
        })
    }

    /// 获取应用配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取 PostgreSQL 连接池
    pub fn postgres_pool(&self) -> PgPool {
        self.postgres_pool.clone()
    }

    /// 获取 Redis 连接管理器
    pub fn redis_connection_manager(&self) -> ConnectionManager {
        self.redis_conn.clone()
    }

    /// 获取 Redis 缓存（实现 CachePort trait）
    pub fn redis_cache(&self) -> RedisCache {
        RedisCache::new(self.redis_conn.clone())
    }

    /// 获取 Token 服务
    pub fn token_service(&self) -> Arc<TokenService> {
        self.token_service.clone()
    }

    /// 获取 JWT 配置
    pub fn jwt_config(&self) -> &cuba_config::JwtConfig {
        &self.config.jwt
    }

    /// 获取服务器配置
    pub fn server_config(&self) -> &cuba_config::ServerConfig {
        &self.config.server
    }
}
