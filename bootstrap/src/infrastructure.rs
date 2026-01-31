//! 基础设施资源管理
//!
//! 统一管理所有微服务共享的基础设施资源

use std::sync::Arc;
use std::time::Duration;

use cuba_adapter_clickhouse::{
    BatchConfig, BatchWriter, ClickHouseConfig as ChAdapterConfig, ClickHousePool,
    CompressionMethod, ReplicaConfig,
};
use cuba_adapter_kafka::{KafkaEventPublisher, KafkaProducerConfig};
pub use cuba_adapter_postgres::PoolStatus;
use cuba_adapter_postgres::{PostgresConfig, ReadWritePool, create_pool};
use cuba_adapter_redis::{RedisCache, create_connection_manager};
use cuba_auth_core::TokenService;
use cuba_config::AppConfig;
use cuba_errors::AppResult;
use clickhouse::Row;
use redis::aio::ConnectionManager;
use secrecy::ExposeSecret;
use serde::Serialize;
use sqlx::PgPool;
use tracing::info;

use crate::retry::{RetryConfig, with_retry, with_retry_optional};

/// 基础设施资源容器
///
/// 包含所有微服务共享的基础设施资源，由 bootstrap 统一初始化
pub struct Infrastructure {
    /// 应用配置
    config: AppConfig,
    /// PostgreSQL 连接池
    postgres_pool: PgPool,
    /// PostgreSQL 读写分离连接池（可选）
    rw_pool: Option<ReadWritePool>,
    /// Redis 连接管理器
    redis_conn: ConnectionManager,
    /// Token 服务
    token_service: Arc<TokenService>,
    /// Kafka Producer（可选）
    kafka_producer: Option<Arc<KafkaEventPublisher>>,
    /// ClickHouse 连接池（可选）
    clickhouse_pool: Option<Arc<ClickHousePool>>,
}

impl Infrastructure {
    /// 从配置创建基础设施资源（带重试）
    pub async fn from_config(config: AppConfig) -> AppResult<Self> {
        let retry_config = RetryConfig::default();

        // 1. 创建 PostgreSQL 连接池（必需，带重试）
        let pg_config = PostgresConfig::new(config.database.url.expose_secret())
            .with_max_connections(config.database.max_connections);
        let postgres_pool = with_retry(&retry_config, "PostgreSQL connection", || {
            let cfg = pg_config.clone();
            async move { create_pool(&cfg).await }
        })
        .await?;
        info!(
            "PostgreSQL connection pool created (max_connections: {})",
            config.database.max_connections
        );

        // 1.1 创建读库连接池（可选，用于读写分离）
        let rw_pool = if let Some(read_url) = &config.database.read_url {
            let read_config = PostgresConfig::new(read_url.expose_secret())
                .with_max_connections(config.database.read_max_connections);

            match with_retry(&retry_config, "PostgreSQL read replica connection", || {
                let cfg = read_config.clone();
                async move { create_pool(&cfg).await }
            })
            .await
            {
                Ok(read_pool) => {
                    info!(
                        "PostgreSQL read replica pool created (max_connections: {})",
                        config.database.read_max_connections
                    );
                    Some(ReadWritePool::new(postgres_pool.clone(), Some(read_pool)))
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to create read replica pool: {}, using primary for reads",
                        e
                    );
                    Some(ReadWritePool::new(postgres_pool.clone(), None))
                }
            }
        } else {
            info!("Read replica not configured, using primary for all operations");
            Some(ReadWritePool::new(postgres_pool.clone(), None))
        };

        // 2. 创建 Redis 连接（必需，带重试）
        let redis_url = config.redis.url.clone();
        let redis_conn = with_retry(&retry_config, "Redis connection", || {
            let url = redis_url.expose_secret().clone();
            async move { create_connection_manager(&url).await }
        })
        .await?;
        info!("Redis connection created");

        // 3. 创建 TokenService
        let token_service = Arc::new(TokenService::new(
            config.jwt.secret.expose_secret(),
            config.jwt.expires_in as i64,
            config.jwt.refresh_expires_in as i64,
            "cuba-iam".to_string(), // issuer
            "cuba-api".to_string(), // audience
        ));

        // 4. 创建 Kafka Producer（可选，带重试）
        let kafka_producer = if let Some(kafka_config) = &config.kafka {
            let producer_config =
                KafkaProducerConfig::new(&kafka_config.brokers).with_client_id(&config.app_name);
            let result = with_retry_optional(&retry_config, "Kafka producer", || {
                let cfg = producer_config.clone();
                async move { KafkaEventPublisher::new(&cfg) }
            })
            .await;
            if result.is_some() {
                info!("Kafka producer created");
            }
            result.map(Arc::new)
        } else {
            info!("Kafka not configured, skipping");
            None
        };

        // 5. 创建 ClickHouse 连接池（可选）
        let clickhouse_pool = if let Some(ch_config) = &config.clickhouse {
            let ch_adapter_config = Self::build_clickhouse_config(ch_config);
            match ClickHousePool::new(ch_adapter_config) {
                Ok(pool) => {
                    info!(
                        pool_max = ch_config.pool_max,
                        batch_size = ch_config.batch_size,
                        "ClickHouse connection pool created"
                    );
                    Some(Arc::new(pool))
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to create ClickHouse pool: {}, continuing without ClickHouse",
                        e
                    );
                    None
                }
            }
        } else {
            info!("ClickHouse not configured, skipping");
            None
        };

        Ok(Self {
            config,
            postgres_pool,
            rw_pool,
            redis_conn,
            token_service,
            kafka_producer,
            clickhouse_pool,
        })
    }

    /// 构建 ClickHouse 适配器配置
    fn build_clickhouse_config(config: &cuba_config::ClickHouseConfig) -> ChAdapterConfig {
        let compression = match config.compression.to_lowercase().as_str() {
            "zstd" => CompressionMethod::Zstd,
            "none" => CompressionMethod::None,
            _ => CompressionMethod::Lz4,
        };

        let replicas: Vec<ReplicaConfig> = config
            .replicas
            .iter()
            .map(|r| ReplicaConfig {
                url: r.url.clone(),
                weight: r.weight,
            })
            .collect();

        let mut ch_config = ChAdapterConfig::new(
            config.url.expose_secret(),
            &config.database,
        )
        .with_pool(config.pool_min, config.pool_max)
        .with_connection_timeout(Duration::from_secs(config.connection_timeout_secs))
        .with_idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .with_retry(
            config.retry_max_attempts,
            Duration::from_millis(config.retry_initial_delay_ms),
            Duration::from_millis(config.retry_max_delay_ms),
        )
        .with_batch(config.batch_size, Duration::from_secs(config.batch_timeout_secs))
        .with_compression(compression);

        if let Some(user) = &config.user {
            if let Some(password) = &config.password {
                ch_config = ch_config.with_auth(user, password.expose_secret());
            }
        }

        if let Some(cluster_name) = &config.cluster_name {
            ch_config = ch_config.with_cluster(cluster_name, replicas);
        }

        ch_config
    }

    /// 获取应用配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取 PostgreSQL 连接池
    pub fn postgres_pool(&self) -> PgPool {
        self.postgres_pool.clone()
    }

    /// 获取读写分离连接池（如果配置了读库）
    pub fn read_write_pool(&self) -> Option<&ReadWritePool> {
        self.rw_pool.as_ref()
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

    /// 获取 Kafka Producer（如果可用）
    pub fn kafka_producer(&self) -> Option<Arc<KafkaEventPublisher>> {
        self.kafka_producer.clone()
    }

    /// 获取 ClickHouse 连接池（如果可用）
    pub fn clickhouse_pool(&self) -> Option<Arc<ClickHousePool>> {
        self.clickhouse_pool.clone()
    }

    /// 获取 ClickHouse 客户端（兼容旧 API）
    pub fn clickhouse_client(&self) -> Option<&clickhouse::Client> {
        self.clickhouse_pool.as_ref().map(|p| p.first_client())
    }

    /// 创建 ClickHouse 批量写入器
    pub fn clickhouse_batch_writer<T: Row + Serialize + Send + Sync + 'static>(
        &self,
        table: &str,
    ) -> Option<BatchWriter<T>> {
        self.clickhouse_pool.as_ref().map(|pool| {
            let batch_config = BatchConfig::from_clickhouse_config(pool.config());
            BatchWriter::new(pool.clone(), table, batch_config)
        })
    }

    /// 检查 Kafka 是否可用
    pub fn has_kafka(&self) -> bool {
        self.kafka_producer.is_some()
    }

    /// 检查 ClickHouse 是否可用
    pub fn has_clickhouse(&self) -> bool {
        self.clickhouse_pool.is_some()
    }

    /// 获取 ClickHouse 连接池状态
    pub fn clickhouse_pool_status(&self) -> Option<cuba_adapter_clickhouse::PoolStatus> {
        self.clickhouse_pool.as_ref().map(|p| p.status())
    }

    /// 获取 PostgreSQL 连接池状态
    pub fn postgres_pool_status(&self) -> PoolStatus {
        if let Some(rw_pool) = &self.rw_pool {
            // 如果有读写分离配置，返回详细状态
            rw_pool.pool_status()
        } else {
            // 否则返回简单状态
            let pool = &self.postgres_pool;
            PoolStatus {
                write_size: pool.size(),
                write_idle: pool.num_idle() as u32,
                write_active: pool.size() - pool.num_idle() as u32,
                read_size: 0,
                read_idle: 0,
                read_active: 0,
            }
        }
    }

    /// 检查 Redis 连接状态
    ///
    /// 返回 true 表示连接可用
    pub async fn check_redis_connection(&self) -> bool {
        let mut conn = self.redis_conn.clone();
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .is_ok()
    }
}
