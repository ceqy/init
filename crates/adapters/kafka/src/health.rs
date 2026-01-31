//! Kafka 健康检查模块
//!
//! 提供 Kafka broker 连通性检查

use std::time::Duration;

use errors::{AppError, AppResult};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::metadata::Metadata;
use tracing::{debug, error};

use crate::config::KafkaConfig;

/// 健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// 是否健康
    pub healthy: bool,
    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
    /// Broker 数量
    pub broker_count: usize,
    /// Topic 数量
    pub topic_count: usize,
}

/// Broker 信息
#[derive(Debug, Clone)]
pub struct BrokerInfo {
    /// Broker ID
    pub id: i32,
    /// 主机名
    pub host: String,
    /// 端口
    pub port: i32,
}

/// Topic 信息
#[derive(Debug, Clone)]
pub struct TopicInfo {
    /// Topic 名称
    pub name: String,
    /// 分区数量
    pub partition_count: usize,
}

/// Kafka 健康检查器
pub struct KafkaHealthChecker {
    consumer: BaseConsumer,
    timeout: Duration,
}

impl KafkaHealthChecker {
    /// 创建健康检查器
    pub fn new(config: &KafkaConfig) -> AppResult<Self> {
        let mut client_config = ClientConfig::new();

        for (key, value) in config.to_client_config_entries() {
            client_config.set(&key, &value);
        }

        let consumer: BaseConsumer = client_config
            .create()
            .map_err(|e| AppError::internal(format!("Failed to create health checker: {}", e)))?;

        Ok(Self {
            consumer,
            timeout: Duration::from_secs(10),
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

    /// 执行健康检查
    pub fn check(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        match self.consumer.fetch_metadata(None, self.timeout) {
            Ok(metadata) => {
                let latency = start.elapsed().as_millis() as u64;
                let broker_count = metadata.brokers().len();
                let topic_count = metadata.topics().len();

                debug!(
                    latency_ms = latency,
                    brokers = broker_count,
                    topics = topic_count,
                    "Kafka health check passed"
                );

                HealthCheckResult {
                    healthy: broker_count > 0,
                    latency_ms: Some(latency),
                    error: None,
                    broker_count,
                    topic_count,
                }
            }
            Err(e) => {
                error!(error = %e, "Kafka health check failed");
                HealthCheckResult {
                    healthy: false,
                    latency_ms: None,
                    error: Some(e.to_string()),
                    broker_count: 0,
                    topic_count: 0,
                }
            }
        }
    }

    /// 获取集群元数据
    pub fn get_metadata(&self) -> AppResult<ClusterMetadata> {
        let metadata = self
            .consumer
            .fetch_metadata(None, self.timeout)
            .map_err(|e| AppError::internal(format!("Failed to fetch metadata: {}", e)))?;

        Ok(ClusterMetadata::from_rdkafka(&metadata))
    }

    /// 获取指定 topic 的元数据
    pub fn get_topic_metadata(&self, topic: &str) -> AppResult<Option<TopicMetadata>> {
        let metadata = self
            .consumer
            .fetch_metadata(Some(topic), self.timeout)
            .map_err(|e| AppError::internal(format!("Failed to fetch topic metadata: {}", e)))?;

        Ok(metadata
            .topics()
            .iter()
            .find(|t| t.name() == topic)
            .map(|t| TopicMetadata {
                name: t.name().to_string(),
                partitions: t
                    .partitions()
                    .iter()
                    .map(|p| PartitionMetadata {
                        id: p.id(),
                        leader: p.leader(),
                        replicas: p.replicas().to_vec(),
                        isr: p.isr().to_vec(),
                    })
                    .collect(),
            }))
    }

    /// 检查 topic 是否存在
    pub fn topic_exists(&self, topic: &str) -> AppResult<bool> {
        let metadata = self
            .consumer
            .fetch_metadata(Some(topic), self.timeout)
            .map_err(|e| AppError::internal(format!("Failed to check topic: {}", e)))?;

        Ok(metadata
            .topics()
            .iter()
            .any(|t| t.name() == topic && t.error().is_none()))
    }
}

/// 集群元数据
#[derive(Debug, Clone)]
pub struct ClusterMetadata {
    /// Broker 列表
    pub brokers: Vec<BrokerInfo>,
    /// Topic 列表
    pub topics: Vec<TopicInfo>,
    /// 原始 broker ID
    pub orig_broker_id: i32,
}

impl ClusterMetadata {
    fn from_rdkafka(metadata: &Metadata) -> Self {
        Self {
            brokers: metadata
                .brokers()
                .iter()
                .map(|b| BrokerInfo {
                    id: b.id(),
                    host: b.host().to_string(),
                    port: b.port(),
                })
                .collect(),
            topics: metadata
                .topics()
                .iter()
                .map(|t| TopicInfo {
                    name: t.name().to_string(),
                    partition_count: t.partitions().len(),
                })
                .collect(),
            orig_broker_id: metadata.orig_broker_id(),
        }
    }
}

/// Topic 元数据
#[derive(Debug, Clone)]
pub struct TopicMetadata {
    /// Topic 名称
    pub name: String,
    /// 分区列表
    pub partitions: Vec<PartitionMetadata>,
}

/// 分区元数据
#[derive(Debug, Clone)]
pub struct PartitionMetadata {
    /// 分区 ID
    pub id: i32,
    /// Leader broker ID
    pub leader: i32,
    /// 副本 broker IDs
    pub replicas: Vec<i32>,
    /// ISR (In-Sync Replicas) broker IDs
    pub isr: Vec<i32>,
}

/// 简单的健康检查函数
pub fn check_kafka_health(brokers: &str) -> AppResult<HealthCheckResult> {
    let checker = KafkaHealthChecker::from_brokers(brokers)?;
    Ok(checker.check())
}

/// 异步健康检查（在后台线程执行）
pub async fn check_kafka_health_async(brokers: String) -> AppResult<HealthCheckResult> {
    tokio::task::spawn_blocking(move || {
        let checker = KafkaHealthChecker::from_brokers(&brokers)?;
        Ok(checker.check())
    })
    .await
    .map_err(|e| AppError::internal(format!("Health check task failed: {}", e)))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_result() {
        let result = HealthCheckResult {
            healthy: true,
            latency_ms: Some(10),
            error: None,
            broker_count: 3,
            topic_count: 10,
        };

        assert!(result.healthy);
        assert_eq!(result.broker_count, 3);
    }

    #[test]
    #[ignore] // 需要 Kafka 实例
    fn test_health_checker() {
        let checker = KafkaHealthChecker::from_brokers("localhost:9092").unwrap();
        let result = checker.check();
        println!("Health check result: {:?}", result);
    }

    #[test]
    #[ignore] // 需要 Kafka 实例
    fn test_get_metadata() {
        let checker = KafkaHealthChecker::from_brokers("localhost:9092").unwrap();
        let metadata = checker.get_metadata().unwrap();
        println!("Brokers: {:?}", metadata.brokers);
        println!("Topics: {:?}", metadata.topics);
    }
}
