//! Redis 配置模块
//!
//! 提供完整的 Redis 配置，包括连接池、集群、哨兵、重试等设置

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Redis 部署模式
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RedisMode {
    /// 单机模式
    #[default]
    Standalone,
    /// 哨兵模式
    Sentinel,
    /// 集群模式
    Cluster,
}

/// 哨兵配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelConfig {
    /// 哨兵节点列表
    pub nodes: Vec<String>,
    /// 主节点名称
    pub master_name: String,
    /// 密码（可选）
    pub password: Option<String>,
}

/// 集群节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNodeConfig {
    /// 节点地址
    pub address: String,
    /// 权重（用于负载均衡）
    #[serde(default = "default_weight")]
    pub weight: u32,
}

fn default_weight() -> u32 {
    1
}

/// Redis 配置
#[derive(Debug, Clone)]
pub struct RedisConfig {
    // 基础配置
    /// Redis URL（单机模式）
    pub url: String,
    /// 部署模式
    pub mode: RedisMode,
    /// 数据库索引
    pub database: u8,
    /// 密码
    pub password: Option<String>,

    // 连接池配置
    /// 最小连接数
    pub pool_min: u32,
    /// 最大连接数
    pub pool_max: u32,
    /// 连接超时
    pub connection_timeout: Duration,
    /// 空闲超时
    pub idle_timeout: Duration,
    /// 获取连接超时
    pub acquire_timeout: Duration,

    // 重试配置
    /// 最大重试次数
    pub retry_max_attempts: u32,
    /// 初始重试延迟
    pub retry_initial_delay: Duration,
    /// 最大重试延迟
    pub retry_max_delay: Duration,

    // 哨兵配置（可选）
    pub sentinel: Option<SentinelConfig>,

    // 集群配置（可选）
    pub cluster_nodes: Vec<ClusterNodeConfig>,

    // 键前缀
    pub key_prefix: Option<String>,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            mode: RedisMode::default(),
            database: 0,
            password: None,
            pool_min: 1,
            pool_max: 10,
            connection_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300),
            acquire_timeout: Duration::from_secs(30),
            retry_max_attempts: 3,
            retry_initial_delay: Duration::from_millis(100),
            retry_max_delay: Duration::from_secs(5),
            sentinel: None,
            cluster_nodes: Vec::new(),
            key_prefix: None,
        }
    }
}

impl RedisConfig {
    /// 创建新的配置
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// 设置密码
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// 设置数据库索引
    pub fn with_database(mut self, database: u8) -> Self {
        self.database = database;
        self
    }

    /// 设置连接池配置
    pub fn with_pool(mut self, min: u32, max: u32) -> Self {
        self.pool_min = min;
        self.pool_max = max;
        self
    }

    /// 设置连接超时
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// 设置空闲超时
    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// 设置获取连接超时
    pub fn with_acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }

    /// 设置重试配置
    pub fn with_retry(
        mut self,
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        self.retry_max_attempts = max_attempts;
        self.retry_initial_delay = initial_delay;
        self.retry_max_delay = max_delay;
        self
    }

    /// 设置哨兵模式
    pub fn with_sentinel(mut self, sentinel: SentinelConfig) -> Self {
        self.mode = RedisMode::Sentinel;
        self.sentinel = Some(sentinel);
        self
    }

    /// 设置集群模式
    pub fn with_cluster(mut self, nodes: Vec<ClusterNodeConfig>) -> Self {
        self.mode = RedisMode::Cluster;
        self.cluster_nodes = nodes;
        self
    }

    /// 设置键前缀
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// 是否为集群模式
    pub fn is_cluster(&self) -> bool {
        self.mode == RedisMode::Cluster
    }

    /// 是否为哨兵模式
    pub fn is_sentinel(&self) -> bool {
        self.mode == RedisMode::Sentinel
    }

    /// 获取带前缀的键
    pub fn prefixed_key(&self, key: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }
}

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_attempts: u32,
    /// 初始延迟
    pub initial_delay: Duration,
    /// 最大延迟
    pub max_delay: Duration,
    /// 退避乘数
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// 创建新的重试配置
    pub fn new(max_attempts: u32, initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay,
            multiplier: 2.0,
        }
    }

    /// 设置退避乘数
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// 从 RedisConfig 创建
    pub fn from_redis_config(config: &RedisConfig) -> Self {
        Self {
            max_attempts: config.retry_max_attempts,
            initial_delay: config.retry_initial_delay,
            max_delay: config.retry_max_delay,
            multiplier: 2.0,
        }
    }

    /// 计算第 n 次重试的延迟
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms =
            self.initial_delay.as_millis() as f64 * self.multiplier.powi(attempt as i32);
        let capped_delay = (delay_ms as u64).min(self.max_delay.as_millis() as u64);
        Duration::from_millis(capped_delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RedisConfig::default();
        assert_eq!(config.pool_min, 1);
        assert_eq!(config.pool_max, 10);
        assert_eq!(config.mode, RedisMode::Standalone);
        assert_eq!(config.database, 0);
    }

    #[test]
    fn test_config_builder() {
        let config = RedisConfig::new("redis://localhost:6379")
            .with_password("secret")
            .with_database(1)
            .with_pool(2, 20)
            .with_key_prefix("myapp");

        assert_eq!(config.password, Some("secret".to_string()));
        assert_eq!(config.database, 1);
        assert_eq!(config.pool_min, 2);
        assert_eq!(config.pool_max, 20);
        assert_eq!(config.key_prefix, Some("myapp".to_string()));
    }

    #[test]
    fn test_prefixed_key() {
        let config = RedisConfig::new("redis://localhost:6379")
            .with_key_prefix("app");

        assert_eq!(config.prefixed_key("user:123"), "app:user:123");

        let config_no_prefix = RedisConfig::new("redis://localhost:6379");
        assert_eq!(config_no_prefix.prefixed_key("user:123"), "user:123");
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig::new(
            5,
            Duration::from_millis(100),
            Duration::from_secs(5),
        );

        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        // Should be capped at max_delay
        assert_eq!(config.delay_for_attempt(10), Duration::from_secs(5));
    }

    #[test]
    fn test_sentinel_config() {
        let config = RedisConfig::new("redis://localhost:6379")
            .with_sentinel(SentinelConfig {
                nodes: vec!["sentinel1:26379".to_string(), "sentinel2:26379".to_string()],
                master_name: "mymaster".to_string(),
                password: None,
            });

        assert!(config.is_sentinel());
        assert!(!config.is_cluster());
    }

    #[test]
    fn test_cluster_config() {
        let config = RedisConfig::new("redis://localhost:6379")
            .with_cluster(vec![
                ClusterNodeConfig {
                    address: "node1:6379".to_string(),
                    weight: 1,
                },
                ClusterNodeConfig {
                    address: "node2:6379".to_string(),
                    weight: 2,
                },
            ]);

        assert!(config.is_cluster());
        assert!(!config.is_sentinel());
        assert_eq!(config.cluster_nodes.len(), 2);
    }
}
