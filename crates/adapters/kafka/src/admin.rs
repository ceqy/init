//! Kafka Admin 模块
//!
//! 提供 Topic 管理功能

use std::collections::HashMap;
use std::time::Duration;

use errors::{AppError, AppResult};
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use tracing::{debug, error, info};

use crate::config::KafkaConfig;

/// Topic 配置
#[derive(Debug, Clone)]
pub struct TopicConfig {
    /// Topic 名称
    pub name: String,
    /// 分区数量
    pub num_partitions: i32,
    /// 副本因子
    pub replication_factor: i32,
    /// 额外配置
    pub config: HashMap<String, String>,
}

impl TopicConfig {
    pub fn new(name: impl Into<String>, num_partitions: i32, replication_factor: i32) -> Self {
        Self {
            name: name.into(),
            num_partitions,
            replication_factor,
            config: HashMap::new(),
        }
    }

    /// 设置保留时间（毫秒）
    pub fn with_retention_ms(mut self, ms: i64) -> Self {
        self.config
            .insert("retention.ms".to_string(), ms.to_string());
        self
    }

    /// 设置保留大小（字节）
    pub fn with_retention_bytes(mut self, bytes: i64) -> Self {
        self.config
            .insert("retention.bytes".to_string(), bytes.to_string());
        self
    }

    /// 设置清理策略
    pub fn with_cleanup_policy(mut self, policy: CleanupPolicy) -> Self {
        self.config
            .insert("cleanup.policy".to_string(), policy.as_str().to_string());
        self
    }

    /// 设置压缩类型
    pub fn with_compression(mut self, compression: &str) -> Self {
        self.config
            .insert("compression.type".to_string(), compression.to_string());
        self
    }

    /// 设置最小 ISR
    pub fn with_min_isr(mut self, min_isr: i32) -> Self {
        self.config
            .insert("min.insync.replicas".to_string(), min_isr.to_string());
        self
    }

    /// 添加自定义配置
    pub fn with_config(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }
}

/// 清理策略
#[derive(Debug, Clone)]
pub enum CleanupPolicy {
    Delete,
    Compact,
    CompactDelete,
}

impl CleanupPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            CleanupPolicy::Delete => "delete",
            CleanupPolicy::Compact => "compact",
            CleanupPolicy::CompactDelete => "compact,delete",
        }
    }
}

/// Kafka Admin 客户端
pub struct KafkaAdmin {
    admin: AdminClient<DefaultClientContext>,
    timeout: Duration,
}

impl KafkaAdmin {
    /// 创建 Admin 客户端
    pub fn new(config: &KafkaConfig) -> AppResult<Self> {
        let mut client_config = ClientConfig::new();

        for (key, value) in config.to_client_config_entries() {
            client_config.set(&key, &value);
        }

        let admin: AdminClient<DefaultClientContext> = client_config
            .create()
            .map_err(|e| AppError::internal(format!("Failed to create admin client: {}", e)))?;

        Ok(Self {
            admin,
            timeout: Duration::from_secs(30),
        })
    }

    /// 从 broker 地址创建
    pub fn from_brokers(brokers: &str) -> AppResult<Self> {
        let config = KafkaConfig::new(brokers);
        Self::new(&config)
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 创建 Topic
    pub async fn create_topic(&self, topic_config: &TopicConfig) -> AppResult<()> {
        let new_topic = NewTopic::new(
            &topic_config.name,
            topic_config.num_partitions,
            TopicReplication::Fixed(topic_config.replication_factor),
        );

        let opts = AdminOptions::new().operation_timeout(Some(self.timeout));

        let results: Vec<Result<String, (String, rdkafka::error::RDKafkaErrorCode)>> = self
            .admin
            .create_topics(&[new_topic], &opts)
            .await
            .map_err(|e| AppError::internal(format!("Failed to create topic: {}", e)))?;

        for result in results {
            match result {
                Ok(name) => {
                    info!(topic = %name, "Topic created successfully");
                }
                Err((name, err)) => {
                    // 忽略 "topic already exists" 错误
                    if format!("{:?}", err).contains("TopicAlreadyExists") {
                        debug!(topic = %name, "Topic already exists");
                    } else {
                        error!(topic = %name, error = ?err, "Failed to create topic");
                        return Err(AppError::internal(format!(
                            "Failed to create topic {}: {:?}",
                            name, err
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// 批量创建 Topics
    pub async fn create_topics(&self, topics: &[TopicConfig]) -> AppResult<()> {
        for topic in topics {
            self.create_topic(topic).await?;
        }
        Ok(())
    }

    /// 删除 Topic
    pub async fn delete_topic(&self, topic: &str) -> AppResult<()> {
        let opts = AdminOptions::new().operation_timeout(Some(self.timeout));

        let results: Vec<Result<String, (String, rdkafka::error::RDKafkaErrorCode)>> = self
            .admin
            .delete_topics(&[topic], &opts)
            .await
            .map_err(|e| AppError::internal(format!("Failed to delete topic: {}", e)))?;

        for result in results {
            match result {
                Ok(name) => {
                    info!(topic = %name, "Topic deleted successfully");
                }
                Err((name, err)) => {
                    error!(topic = %name, error = ?err, "Failed to delete topic");
                    return Err(AppError::internal(format!(
                        "Failed to delete topic {}: {:?}",
                        name, err
                    )));
                }
            }
        }

        Ok(())
    }

    /// 批量删除 Topics
    pub async fn delete_topics(&self, topics: &[&str]) -> AppResult<()> {
        let opts = AdminOptions::new().operation_timeout(Some(self.timeout));

        let results: Vec<Result<String, (String, rdkafka::error::RDKafkaErrorCode)>> = self
            .admin
            .delete_topics(topics, &opts)
            .await
            .map_err(|e| AppError::internal(format!("Failed to delete topics: {}", e)))?;

        for result in results {
            match result {
                Ok(name) => {
                    info!(topic = %name, "Topic deleted successfully");
                }
                Err((name, err)) => {
                    error!(topic = %name, error = ?err, "Failed to delete topic");
                }
            }
        }

        Ok(())
    }

    /// 增加分区数量
    pub async fn add_partitions(&self, topic: &str, new_partition_count: i32) -> AppResult<()> {
        use rdkafka::admin::NewPartitions;

        let new_partitions = NewPartitions::new(topic, new_partition_count as usize);
        let opts = AdminOptions::new().operation_timeout(Some(self.timeout));

        let results: Vec<Result<String, (String, rdkafka::error::RDKafkaErrorCode)>> = self
            .admin
            .create_partitions(&[new_partitions], &opts)
            .await
            .map_err(|e| AppError::internal(format!("Failed to add partitions: {}", e)))?;

        for result in results {
            match result {
                Ok(name) => {
                    info!(
                        topic = %name,
                        partitions = new_partition_count,
                        "Partitions added successfully"
                    );
                }
                Err((name, err)) => {
                    error!(topic = %name, error = ?err, "Failed to add partitions");
                    return Err(AppError::internal(format!(
                        "Failed to add partitions to {}: {:?}",
                        name, err
                    )));
                }
            }
        }

        Ok(())
    }
}

/// 创建标准 Topic 配置的辅助函数
pub fn standard_topic(name: &str, partitions: i32) -> TopicConfig {
    TopicConfig::new(name, partitions, 1)
        .with_retention_ms(7 * 24 * 60 * 60 * 1000) // 7 天
        .with_cleanup_policy(CleanupPolicy::Delete)
}

/// 创建紧凑 Topic 配置的辅助函数
pub fn compacted_topic(name: &str, partitions: i32) -> TopicConfig {
    TopicConfig::new(name, partitions, 1).with_cleanup_policy(CleanupPolicy::Compact)
}

/// 创建 DLQ Topic 配置的辅助函数
pub fn dlq_topic(original_topic: &str, suffix: &str) -> TopicConfig {
    TopicConfig::new(format!("{}{}", original_topic, suffix), 1, 1)
        .with_retention_ms(30 * 24 * 60 * 60 * 1000) // 30 天
        .with_cleanup_policy(CleanupPolicy::Delete)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_config() {
        let config = TopicConfig::new("test-topic", 3, 2)
            .with_retention_ms(86400000)
            .with_cleanup_policy(CleanupPolicy::Compact)
            .with_min_isr(2);

        assert_eq!(config.name, "test-topic");
        assert_eq!(config.num_partitions, 3);
        assert_eq!(config.replication_factor, 2);
        assert_eq!(
            config.config.get("retention.ms"),
            Some(&"86400000".to_string())
        );
        assert_eq!(
            config.config.get("cleanup.policy"),
            Some(&"compact".to_string())
        );
    }

    #[test]
    fn test_standard_topic() {
        let config = standard_topic("events", 6);
        assert_eq!(config.num_partitions, 6);
        assert!(config.config.contains_key("retention.ms"));
    }

    #[tokio::test]
    #[ignore] // 需要 Kafka 实例
    async fn test_create_topic() {
        let admin = KafkaAdmin::from_brokers("localhost:9092").unwrap();
        let config = TopicConfig::new("test-topic", 3, 1);
        admin.create_topic(&config).await.unwrap();
    }
}
