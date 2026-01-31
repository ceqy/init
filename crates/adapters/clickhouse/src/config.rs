//! ClickHouse 配置模块
//!
//! 提供完整的 ClickHouse 配置，包括连接池、重试、批量写入、集群等设置

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 压缩方法
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionMethod {
    /// 不压缩
    None,
    /// LZ4 压缩（默认，速度快）
    #[default]
    Lz4,
    /// Zstd 压缩（压缩率高）
    Zstd,
}

/// 副本配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaConfig {
    /// 副本 URL
    pub url: String,
    /// 权重（用于负载均衡）
    #[serde(default = "default_weight")]
    pub weight: u32,
}

fn default_weight() -> u32 {
    1
}

/// ClickHouse 配置
#[derive(Debug, Clone)]
pub struct ClickHouseConfig {
    // 基础配置
    /// ClickHouse URL
    pub url: String,
    /// 数据库名称
    pub database: String,
    /// 用户名
    pub user: Option<String>,
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

    // 重试配置
    /// 最大重试次数
    pub retry_max_attempts: u32,
    /// 初始重试延迟
    pub retry_initial_delay: Duration,
    /// 最大重试延迟
    pub retry_max_delay: Duration,

    // 批量写入配置
    /// 批量大小
    pub batch_size: usize,
    /// 批量超时
    pub batch_timeout: Duration,

    // 集群配置（可选）
    /// 集群名称
    pub cluster_name: Option<String>,
    /// 副本列表
    pub replicas: Vec<ReplicaConfig>,

    // 压缩配置
    /// 压缩方法
    pub compression: CompressionMethod,
}

impl Default for ClickHouseConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8123".to_string(),
            database: "default".to_string(),
            user: None,
            password: None,
            pool_min: 1,
            pool_max: 10,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            retry_max_attempts: 3,
            retry_initial_delay: Duration::from_millis(100),
            retry_max_delay: Duration::from_secs(10),
            batch_size: 10000,
            batch_timeout: Duration::from_secs(5),
            cluster_name: None,
            replicas: Vec::new(),
            compression: CompressionMethod::default(),
        }
    }
}

impl ClickHouseConfig {
    /// 创建新的配置
    pub fn new(url: impl Into<String>, database: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            database: database.into(),
            ..Default::default()
        }
    }

    /// 设置认证信息
    pub fn with_auth(mut self, user: impl Into<String>, password: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self.password = Some(password.into());
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

    /// 设置批量写入配置
    pub fn with_batch(mut self, size: usize, timeout: Duration) -> Self {
        self.batch_size = size;
        self.batch_timeout = timeout;
        self
    }

    /// 设置集群配置
    pub fn with_cluster(mut self, name: impl Into<String>, replicas: Vec<ReplicaConfig>) -> Self {
        self.cluster_name = Some(name.into());
        self.replicas = replicas;
        self
    }

    /// 设置压缩方法
    pub fn with_compression(mut self, compression: CompressionMethod) -> Self {
        self.compression = compression;
        self
    }

    /// 获取所有节点 URL（包括主节点和副本）
    pub fn all_urls(&self) -> Vec<&str> {
        let mut urls = vec![self.url.as_str()];
        for replica in &self.replicas {
            urls.push(replica.url.as_str());
        }
        urls
    }

    /// 是否为集群模式
    pub fn is_cluster(&self) -> bool {
        self.cluster_name.is_some() && !self.replicas.is_empty()
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
            max_delay: Duration::from_secs(10),
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

    /// 从 ClickHouseConfig 创建
    pub fn from_clickhouse_config(config: &ClickHouseConfig) -> Self {
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

/// 批量写入配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 批量大小
    pub size: usize,
    /// 批量超时
    pub timeout: Duration,
    /// 最大内存使用（字节）
    pub max_memory: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            size: 10000,
            timeout: Duration::from_secs(5),
            max_memory: 100 * 1024 * 1024, // 100MB
        }
    }
}

impl BatchConfig {
    /// 创建新的批量配置
    pub fn new(size: usize, timeout: Duration) -> Self {
        Self {
            size,
            timeout,
            ..Default::default()
        }
    }

    /// 设置最大内存
    pub fn with_max_memory(mut self, max_memory: usize) -> Self {
        self.max_memory = max_memory;
        self
    }

    /// 从 ClickHouseConfig 创建
    pub fn from_clickhouse_config(config: &ClickHouseConfig) -> Self {
        Self {
            size: config.batch_size,
            timeout: config.batch_timeout,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClickHouseConfig::default();
        assert_eq!(config.pool_min, 1);
        assert_eq!(config.pool_max, 10);
        assert_eq!(config.batch_size, 10000);
        assert_eq!(config.compression, CompressionMethod::Lz4);
    }

    #[test]
    fn test_config_builder() {
        let config = ClickHouseConfig::new("http://localhost:8123", "test_db")
            .with_auth("user", "pass")
            .with_pool(2, 20)
            .with_batch(5000, Duration::from_secs(10))
            .with_compression(CompressionMethod::Zstd);

        assert_eq!(config.database, "test_db");
        assert_eq!(config.user, Some("user".to_string()));
        assert_eq!(config.pool_min, 2);
        assert_eq!(config.pool_max, 20);
        assert_eq!(config.batch_size, 5000);
        assert_eq!(config.compression, CompressionMethod::Zstd);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig::new(
            5,
            Duration::from_millis(100),
            Duration::from_secs(10),
        );

        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(800));
        // Should be capped at max_delay
        assert_eq!(config.delay_for_attempt(10), Duration::from_secs(10));
    }

    #[test]
    fn test_cluster_config() {
        let config = ClickHouseConfig::new("http://localhost:8123", "test_db")
            .with_cluster(
                "test_cluster",
                vec![
                    ReplicaConfig {
                        url: "http://replica1:8123".to_string(),
                        weight: 1,
                    },
                    ReplicaConfig {
                        url: "http://replica2:8123".to_string(),
                        weight: 2,
                    },
                ],
            );

        assert!(config.is_cluster());
        assert_eq!(config.all_urls().len(), 3);
    }
}
