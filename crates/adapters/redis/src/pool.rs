//! Redis 连接池管理
//!
//! 提供连接池、并发控制、健康状态跟踪

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use errors::{AppError, AppResult};
use parking_lot::RwLock;
use redis::aio::ConnectionManager;
use redis::Client;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::{debug, info, warn};

use crate::config::RedisConfig;

/// 连接池状态
#[derive(Debug, Clone)]
pub struct PoolStatus {
    /// 总连接数
    pub total_connections: usize,
    /// 活跃连接数
    pub active_connections: usize,
    /// 空闲连接数
    pub idle_connections: usize,
    /// 最大连接数
    pub max_connections: usize,
    /// 等待中的请求数
    pub waiting_requests: usize,
    /// 是否健康
    pub healthy: bool,
}

/// Redis 连接池
pub struct RedisPool {
    /// 连接管理器列表
    connections: Vec<ConnectionManager>,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 健康状态
    healthy: Arc<RwLock<bool>>,
    /// 当前活跃连接计数
    active_count: Arc<AtomicUsize>,
    /// 轮询索引
    round_robin_index: AtomicUsize,
    /// 配置
    config: RedisConfig,
}

impl RedisPool {
    /// 创建新的连接池
    pub async fn new(config: RedisConfig) -> AppResult<Self> {
        let pool_size = config.pool_max as usize;
        let mut connections = Vec::with_capacity(pool_size);

        // 创建 Redis 客户端
        let client = Client::open(config.url.as_str())
            .map_err(|e| AppError::internal(format!("Failed to create Redis client: {}", e)))?;

        // 创建多个连接管理器
        for i in 0..pool_size {
            let conn = ConnectionManager::new(client.clone())
                .await
                .map_err(|e| {
                    AppError::internal(format!(
                        "Failed to create Redis connection {}: {}",
                        i, e
                    ))
                })?;
            connections.push(conn);
        }

        info!(
            pool_size = pool_size,
            url = %config.url,
            "Redis connection pool created"
        );

        let semaphore = Arc::new(Semaphore::new(pool_size));

        Ok(Self {
            connections,
            semaphore,
            healthy: Arc::new(RwLock::new(true)),
            active_count: Arc::new(AtomicUsize::new(0)),
            round_robin_index: AtomicUsize::new(0),
            config,
        })
    }

    /// 获取一个连接
    pub async fn get(&self) -> AppResult<PooledConnection<'_>> {
        // 获取信号量许可
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AppError::resource_exhausted("Redis connection pool exhausted"))?;

        // 选择一个连接（轮询）
        let index = self.round_robin_index.fetch_add(1, Ordering::SeqCst) % self.connections.len();
        self.active_count.fetch_add(1, Ordering::SeqCst);

        Ok(PooledConnection {
            pool: self,
            conn_index: index,
            _permit: permit,
        })
    }

    /// 尝试获取连接（非阻塞）
    pub fn try_get(&self) -> AppResult<Option<PooledConnection<'_>>> {
        match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                let index =
                    self.round_robin_index.fetch_add(1, Ordering::SeqCst) % self.connections.len();
                self.active_count.fetch_add(1, Ordering::SeqCst);
                Ok(Some(PooledConnection {
                    pool: self,
                    conn_index: index,
                    _permit: permit,
                }))
            }
            Err(_) => Ok(None),
        }
    }

    /// 标记为不健康
    pub fn mark_unhealthy(&self) {
        let mut healthy = self.healthy.write();
        *healthy = false;
        warn!("Redis pool marked as unhealthy");
    }

    /// 标记为健康
    pub fn mark_healthy(&self) {
        let mut healthy = self.healthy.write();
        *healthy = true;
        debug!("Redis pool marked as healthy");
    }

    /// 检查是否健康
    pub fn is_healthy(&self) -> bool {
        *self.healthy.read()
    }

    /// 获取连接池状态
    pub fn status(&self) -> PoolStatus {
        let active = self.active_count.load(Ordering::SeqCst);
        let max = self.config.pool_max as usize;
        let total = self.connections.len();

        PoolStatus {
            total_connections: total,
            active_connections: active,
            idle_connections: total.saturating_sub(active),
            max_connections: max,
            waiting_requests: active.saturating_sub(max),
            healthy: self.is_healthy(),
        }
    }

    /// 获取配置
    pub fn config(&self) -> &RedisConfig {
        &self.config
    }

    /// 获取原始连接（用于特殊操作）
    pub fn connection(&self, index: usize) -> Option<&ConnectionManager> {
        self.connections.get(index)
    }

    /// 获取第一个连接
    pub fn first_connection(&self) -> &ConnectionManager {
        &self.connections[0]
    }

    /// 连接数量
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

/// 池化的连接
pub struct PooledConnection<'a> {
    pool: &'a RedisPool,
    conn_index: usize,
    _permit: OwnedSemaphorePermit,
}

impl<'a> PooledConnection<'a> {
    /// 获取底层连接
    pub fn connection(&self) -> ConnectionManager {
        self.pool.connections[self.conn_index].clone()
    }

    /// 获取连接索引
    pub fn index(&self) -> usize {
        self.conn_index
    }

    /// 标记连接池为不健康
    pub fn mark_unhealthy(&self) {
        self.pool.mark_unhealthy();
    }
}

impl Drop for PooledConnection<'_> {
    fn drop(&mut self) {
        self.pool.active_count.fetch_sub(1, Ordering::SeqCst);
    }
}

impl std::ops::Deref for PooledConnection<'_> {
    type Target = ConnectionManager;

    fn deref(&self) -> &Self::Target {
        &self.pool.connections[self.conn_index]
    }
}

/// 创建 Redis 连接管理器（兼容旧 API）
pub async fn create_connection_manager(url: &str) -> AppResult<ConnectionManager> {
    let client = Client::open(url)
        .map_err(|e| AppError::internal(format!("Failed to create Redis client: {}", e)))?;

    ConnectionManager::new(client).await.map_err(|e| {
        AppError::internal(format!("Failed to create Redis connection manager: {}", e))
    })
}

/// 检查 Redis 连接
pub async fn check_connection(conn: &mut ConnectionManager) -> AppResult<()> {
    redis::cmd("PING")
        .query_async::<String>(conn)
        .await
        .map_err(|e| AppError::internal(format!("Redis health check failed: {}", e)))?;
    Ok(())
}

/// 检查连接池健康状态
pub async fn check_pool_health(pool: &RedisPool) -> AppResult<()> {
    let conn = pool.get().await?;
    let mut conn_manager = conn.connection();
    check_connection(&mut conn_manager).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要 Redis 实例
    async fn test_pool_creation() {
        let config = RedisConfig::new("redis://127.0.0.1:6379").with_pool(1, 5);
        let pool = RedisPool::new(config).await.unwrap();
        let status = pool.status();

        assert_eq!(status.total_connections, 5);
        assert_eq!(status.active_connections, 0);
        assert!(status.healthy);
    }

    #[tokio::test]
    #[ignore] // 需要 Redis 实例
    async fn test_pool_get_connection() {
        let config = RedisConfig::new("redis://127.0.0.1:6379").with_pool(1, 5);
        let pool = RedisPool::new(config).await.unwrap();

        let conn = pool.get().await.unwrap();
        let status = pool.status();
        assert_eq!(status.active_connections, 1);

        drop(conn);
        let status = pool.status();
        assert_eq!(status.active_connections, 0);
    }

    #[test]
    fn test_pool_status() {
        let status = PoolStatus {
            total_connections: 10,
            active_connections: 3,
            idle_connections: 7,
            max_connections: 10,
            waiting_requests: 0,
            healthy: true,
        };

        assert_eq!(status.idle_connections, 7);
        assert!(status.healthy);
    }
}
