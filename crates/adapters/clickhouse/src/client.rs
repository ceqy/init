//! ClickHouse 连接池管理
//!
//! 提供多节点支持、并发控制、健康状态跟踪

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use clickhouse::Client;
use errors::{AppError, AppResult};
use parking_lot::RwLock;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::{debug, warn};

use crate::config::ClickHouseConfig;

/// 连接池状态
#[derive(Debug, Clone)]
pub struct PoolStatus {
    /// 总节点数
    pub total_nodes: usize,
    /// 健康节点数
    pub healthy_nodes: usize,
    /// 当前活跃连接数
    pub active_connections: usize,
    /// 最大连接数
    pub max_connections: usize,
    /// 等待中的请求数
    pub waiting_requests: usize,
}

/// ClickHouse 连接池
pub struct ClickHousePool {
    /// 客户端列表（支持多节点）
    clients: Vec<Client>,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 健康状态
    healthy: Arc<RwLock<Vec<bool>>>,
    /// 当前活跃连接计数
    active_count: Arc<AtomicUsize>,
    /// 轮询索引
    round_robin_index: AtomicUsize,
    /// 配置
    config: ClickHouseConfig,
}

impl ClickHousePool {
    /// 创建新的连接池
    pub fn new(config: ClickHouseConfig) -> AppResult<Self> {
        let urls = config.all_urls();
        let mut clients = Vec::with_capacity(urls.len());

        for url in urls {
            let mut client = Client::default()
                .with_url(url)
                .with_database(&config.database);

            if let (Some(user), Some(password)) = (&config.user, &config.password) {
                client = client.with_user(user).with_password(password);
            }

            clients.push(client);
        }

        let node_count = clients.len();
        let healthy = Arc::new(RwLock::new(vec![true; node_count]));
        let semaphore = Arc::new(Semaphore::new(config.pool_max as usize));

        Ok(Self {
            clients,
            semaphore,
            healthy,
            active_count: Arc::new(AtomicUsize::new(0)),
            round_robin_index: AtomicUsize::new(0),
            config,
        })
    }

    /// 获取一个连接
    pub async fn get(&self) -> AppResult<PooledClient<'_>> {
        // 获取信号量许可
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AppError::resource_exhausted("Connection pool exhausted"))?;

        // 选择一个健康的节点
        let client_index = self.select_healthy_node()?;
        self.active_count.fetch_add(1, Ordering::SeqCst);

        Ok(PooledClient {
            pool: self,
            client_index,
            _permit: permit,
        })
    }

    /// 尝试获取连接（非阻塞）
    pub fn try_get(&self) -> AppResult<Option<PooledClient<'_>>> {
        match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                let client_index = self.select_healthy_node()?;
                self.active_count.fetch_add(1, Ordering::SeqCst);
                Ok(Some(PooledClient {
                    pool: self,
                    client_index,
                    _permit: permit,
                }))
            }
            Err(_) => Ok(None),
        }
    }

    /// 选择一个健康的节点（轮询）
    fn select_healthy_node(&self) -> AppResult<usize> {
        let healthy = self.healthy.read();
        let node_count = self.clients.len();

        // 尝试找到一个健康的节点
        for _ in 0..node_count {
            let index = self.round_robin_index.fetch_add(1, Ordering::SeqCst) % node_count;
            if healthy[index] {
                return Ok(index);
            }
        }

        // 如果没有健康节点，返回第一个节点（可能会失败，但让调用者处理）
        warn!("No healthy ClickHouse nodes available, using first node");
        Ok(0)
    }

    /// 标记节点为不健康
    pub fn mark_unhealthy(&self, index: usize) {
        let mut healthy = self.healthy.write();
        if index < healthy.len() {
            healthy[index] = false;
            warn!(node_index = index, "ClickHouse node marked as unhealthy");
        }
    }

    /// 标记节点为健康
    pub fn mark_healthy(&self, index: usize) {
        let mut healthy = self.healthy.write();
        if index < healthy.len() {
            healthy[index] = true;
            debug!(node_index = index, "ClickHouse node marked as healthy");
        }
    }

    /// 获取连接池状态
    pub fn status(&self) -> PoolStatus {
        let healthy = self.healthy.read();
        let healthy_count = healthy.iter().filter(|&&h| h).count();
        let active = self.active_count.load(Ordering::SeqCst);
        let max = self.config.pool_max as usize;

        PoolStatus {
            total_nodes: self.clients.len(),
            healthy_nodes: healthy_count,
            active_connections: active,
            max_connections: max,
            waiting_requests: active.saturating_sub(max),
        }
    }

    /// 获取配置
    pub fn config(&self) -> &ClickHouseConfig {
        &self.config
    }

    /// 获取原始客户端（用于健康检查等）
    pub fn client(&self, index: usize) -> Option<&Client> {
        self.clients.get(index)
    }

    /// 获取第一个客户端
    pub fn first_client(&self) -> &Client {
        &self.clients[0]
    }

    /// 节点数量
    pub fn node_count(&self) -> usize {
        self.clients.len()
    }
}

/// 池化的客户端连接
pub struct PooledClient<'a> {
    pool: &'a ClickHousePool,
    client_index: usize,
    _permit: OwnedSemaphorePermit,
}

impl<'a> PooledClient<'a> {
    /// 获取底层客户端
    pub fn client(&self) -> &Client {
        &self.pool.clients[self.client_index]
    }

    /// 获取节点索引
    pub fn node_index(&self) -> usize {
        self.client_index
    }

    /// 标记当前节点为不健康
    pub fn mark_unhealthy(&self) {
        self.pool.mark_unhealthy(self.client_index);
    }
}

impl Drop for PooledClient<'_> {
    fn drop(&mut self) {
        self.pool.active_count.fetch_sub(1, Ordering::SeqCst);
    }
}

impl std::ops::Deref for PooledClient<'_> {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        self.client()
    }
}

/// 检查 ClickHouse 连接
pub async fn check_connection(client: &Client) -> AppResult<()> {
    client
        .query("SELECT 1")
        .fetch_one::<u8>()
        .await
        .map_err(|e| AppError::database(format!("ClickHouse health check failed: {}", e)))?;
    Ok(())
}

/// 检查连接池健康状态
pub async fn check_pool_health(pool: &ClickHousePool) -> AppResult<()> {
    let client = pool.get().await?;
    check_connection(client.client()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_status() {
        let config = ClickHouseConfig::new("http://localhost:8123", "test");
        let pool = ClickHousePool::new(config).unwrap();
        let status = pool.status();

        assert_eq!(status.total_nodes, 1);
        assert_eq!(status.healthy_nodes, 1);
        assert_eq!(status.active_connections, 0);
        assert_eq!(status.max_connections, 10);
    }

    #[test]
    fn test_mark_unhealthy() {
        let config = ClickHouseConfig::new("http://localhost:8123", "test");
        let pool = ClickHousePool::new(config).unwrap();

        pool.mark_unhealthy(0);
        let status = pool.status();
        assert_eq!(status.healthy_nodes, 0);

        pool.mark_healthy(0);
        let status = pool.status();
        assert_eq!(status.healthy_nodes, 1);
    }
}
