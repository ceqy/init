//! Redis 集群支持模块
//!
//! 提供 Redis Cluster 连接和操作支持

use std::sync::Arc;
use std::time::Duration;

use errors::{AppError, AppResult};
use redis::cluster::ClusterClient;
use redis::cluster_async::ClusterConnection;
use redis::AsyncCommands;
use tracing::{debug, error, info};

use crate::config::{ClusterNodeConfig, RedisConfig};

/// Redis 集群连接
pub struct RedisCluster {
    client: ClusterClient,
    config: RedisConfig,
}

impl RedisCluster {
    /// 从配置创建集群连接
    pub fn new(config: RedisConfig) -> AppResult<Self> {
        if config.cluster_nodes.is_empty() {
            return Err(AppError::validation("Cluster nodes cannot be empty"));
        }

        let nodes: Vec<String> = config
            .cluster_nodes
            .iter()
            .map(|n| n.address.clone())
            .collect();

        let client = ClusterClient::new(nodes)
            .map_err(|e| AppError::internal(format!("Failed to create cluster client: {}", e)))?;

        info!(
            nodes = config.cluster_nodes.len(),
            "Redis cluster client created"
        );

        Ok(Self { client, config })
    }

    /// 从节点地址列表创建
    pub fn from_nodes(nodes: Vec<String>) -> AppResult<Self> {
        let cluster_nodes: Vec<ClusterNodeConfig> = nodes
            .into_iter()
            .map(|address| ClusterNodeConfig { address, weight: 1 })
            .collect();

        let config = RedisConfig::default().with_cluster(cluster_nodes);
        Self::new(config)
    }

    /// 获取异步连接
    pub async fn get_connection(&self) -> AppResult<ClusterConnection> {
        self.client
            .get_async_connection()
            .await
            .map_err(|e| AppError::internal(format!("Failed to get cluster connection: {}", e)))
    }

    /// 获取配置
    pub fn config(&self) -> &RedisConfig {
        &self.config
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.config.cluster_nodes.len()
    }
}

/// 集群缓存操作
pub struct ClusterCache {
    cluster: Arc<RedisCluster>,
    key_prefix: Option<String>,
}

impl ClusterCache {
    /// 创建集群缓存
    pub fn new(cluster: Arc<RedisCluster>) -> Self {
        Self {
            cluster,
            key_prefix: None,
        }
    }

    /// 设置键前缀
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// 获取带前缀的键
    fn prefixed_key(&self, key: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }

    /// 获取值
    pub async fn get(&self, key: &str) -> AppResult<Option<String>> {
        let key = self.prefixed_key(key);
        let mut conn = self.cluster.get_connection().await?;

        let result: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| AppError::internal(format!("Cluster get failed: {}", e)))?;
        Ok(result)
    }

    /// 设置值
    pub async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> AppResult<()> {
        let key = self.prefixed_key(key);
        let mut conn = self.cluster.get_connection().await?;

        match ttl {
            Some(duration) => {
                let _: () = conn
                    .set_ex(&key, value, duration.as_secs())
                    .await
                    .map_err(|e| AppError::internal(format!("Cluster set failed: {}", e)))?;
            }
            None => {
                let _: () = conn
                    .set(&key, value)
                    .await
                    .map_err(|e| AppError::internal(format!("Cluster set failed: {}", e)))?;
            }
        }
        Ok(())
    }

    /// 删除键
    pub async fn delete(&self, key: &str) -> AppResult<()> {
        let key = self.prefixed_key(key);
        let mut conn = self.cluster.get_connection().await?;

        let _: () = conn
            .del(&key)
            .await
            .map_err(|e| AppError::internal(format!("Cluster delete failed: {}", e)))?;
        Ok(())
    }

    /// 检查键是否存在
    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        let key = self.prefixed_key(key);
        let mut conn = self.cluster.get_connection().await?;

        let result: bool = conn
            .exists(&key)
            .await
            .map_err(|e| AppError::internal(format!("Cluster exists failed: {}", e)))?;
        Ok(result)
    }

    /// 设置过期时间
    pub async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()> {
        let key = self.prefixed_key(key);
        let mut conn = self.cluster.get_connection().await?;

        let _: () = conn
            .expire(&key, ttl.as_secs() as i64)
            .await
            .map_err(|e| AppError::internal(format!("Cluster expire failed: {}", e)))?;
        Ok(())
    }

    /// 递增
    pub async fn incr(&self, key: &str) -> AppResult<i64> {
        let key = self.prefixed_key(key);
        let mut conn = self.cluster.get_connection().await?;

        let result: i64 = conn
            .incr(&key, 1)
            .await
            .map_err(|e| AppError::internal(format!("Cluster incr failed: {}", e)))?;
        Ok(result)
    }

    /// 递减
    pub async fn decr(&self, key: &str) -> AppResult<i64> {
        let key = self.prefixed_key(key);
        let mut conn = self.cluster.get_connection().await?;

        let result: i64 = conn
            .decr(&key, 1)
            .await
            .map_err(|e| AppError::internal(format!("Cluster decr failed: {}", e)))?;
        Ok(result)
    }
}

/// 集群健康检查
pub async fn check_cluster_health(cluster: &RedisCluster) -> AppResult<ClusterHealthResult> {
    let start = std::time::Instant::now();

    match cluster.get_connection().await {
        Ok(mut conn) => {
            let result: Result<String, redis::RedisError> =
                redis::cmd("PING").query_async(&mut conn).await;

            match result {
                Ok(_) => {
                    let latency = start.elapsed().as_millis() as u64;
                    debug!(latency_ms = latency, "Cluster health check passed");
                    Ok(ClusterHealthResult {
                        healthy: true,
                        latency_ms: Some(latency),
                        error: None,
                        node_count: cluster.node_count(),
                    })
                }
                Err(e) => {
                    error!(error = %e, "Cluster health check failed");
                    Ok(ClusterHealthResult {
                        healthy: false,
                        latency_ms: None,
                        error: Some(e.to_string()),
                        node_count: cluster.node_count(),
                    })
                }
            }
        }
        Err(e) => Ok(ClusterHealthResult {
            healthy: false,
            latency_ms: None,
            error: Some(e.to_string()),
            node_count: cluster.node_count(),
        }),
    }
}

/// 集群健康检查结果
#[derive(Debug, Clone)]
pub struct ClusterHealthResult {
    /// 是否健康
    pub healthy: bool,
    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
    /// 节点数量
    pub node_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_health_result() {
        let result = ClusterHealthResult {
            healthy: true,
            latency_ms: Some(5),
            error: None,
            node_count: 3,
        };

        assert!(result.healthy);
        assert_eq!(result.node_count, 3);
    }

    #[tokio::test]
    #[ignore] // 需要 Redis 集群实例
    async fn test_cluster_connection() {
        let nodes = vec![
            "redis://127.0.0.1:7000".to_string(),
            "redis://127.0.0.1:7001".to_string(),
            "redis://127.0.0.1:7002".to_string(),
        ];

        let cluster = RedisCluster::from_nodes(nodes).unwrap();
        let conn = cluster.get_connection().await;
        assert!(conn.is_ok());
    }

    #[tokio::test]
    #[ignore] // 需要 Redis 集群实例
    async fn test_cluster_cache() {
        let nodes = vec![
            "redis://127.0.0.1:7000".to_string(),
            "redis://127.0.0.1:7001".to_string(),
            "redis://127.0.0.1:7002".to_string(),
        ];

        let cluster = Arc::new(RedisCluster::from_nodes(nodes).unwrap());
        let cache = ClusterCache::new(cluster).with_key_prefix("test");

        // 设置值
        cache
            .set("key1", "value1", Some(Duration::from_secs(60)))
            .await
            .unwrap();

        // 获取值
        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // 删除
        cache.delete("key1").await.unwrap();

        // 确认删除
        let value = cache.get("key1").await.unwrap();
        assert!(value.is_none());
    }
}
