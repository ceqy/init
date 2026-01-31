//! CDC 消费者模块
//!
//! 从 Kafka 消费 PostgreSQL 变更事件并写入 ClickHouse

use std::sync::Arc;

use clickhouse::Row;
use adapter_kafka::{KafkaConsumerConfig, KafkaEventConsumer};
use errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::batch::BatchWriter;
use crate::client::ClickHousePool;
use crate::config::BatchConfig;

/// CDC 记录
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct CdcRecord {
    /// 操作类型 (INSERT, UPDATE, DELETE)
    pub operation: String,
    /// 表名
    pub table_name: String,
    /// 主键
    pub primary_key: String,
    /// 变更前数据（JSON）
    pub before: Option<String>,
    /// 变更后数据（JSON）
    pub after: Option<String>,
    /// 事务 ID
    pub transaction_id: Option<String>,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 来源
    pub source: String,
}

/// Debezium CDC 消息格式
#[derive(Debug, Clone, Deserialize)]
pub struct DebeziumMessage {
    pub payload: DebeziumPayload,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DebeziumPayload {
    pub op: String,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
    pub source: DebeziumSource,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DebeziumSource {
    pub table: String,
    #[serde(rename = "txId")]
    pub tx_id: Option<i64>,
    pub db: String,
}

impl CdcRecord {
    /// 从 Debezium 消息创建
    pub fn from_debezium(msg: &DebeziumMessage) -> Self {
        let payload = &msg.payload;
        let operation = match payload.op.as_str() {
            "c" => "INSERT",
            "u" => "UPDATE",
            "d" => "DELETE",
            "r" => "READ", // Snapshot read
            _ => "UNKNOWN",
        }
        .to_string();

        let primary_key = payload
            .after
            .as_ref()
            .or(payload.before.as_ref())
            .and_then(|v| v.get("id"))
            .map(|v| v.to_string())
            .unwrap_or_default();

        Self {
            operation,
            table_name: payload.source.table.clone(),
            primary_key,
            before: payload.before.as_ref().map(|v| v.to_string()),
            after: payload.after.as_ref().map(|v| v.to_string()),
            transaction_id: payload.source.tx_id.map(|id| id.to_string()),
            timestamp: chrono::DateTime::from_timestamp_millis(payload.ts_ms)
                .unwrap_or_else(chrono::Utc::now),
            source: format!("{}.{}", payload.source.db, payload.source.table),
        }
    }
}

/// CDC 同步消费者
pub struct CdcSyncConsumer {
    kafka_consumer: KafkaEventConsumer,
    #[allow(dead_code)]
    clickhouse_pool: Arc<ClickHousePool>,
    batch_writer: Arc<BatchWriter<CdcRecord>>,
    table: String,
}

impl CdcSyncConsumer {
    /// 创建新的 CDC 同步消费者
    pub fn new(
        kafka_config: KafkaConsumerConfig,
        clickhouse_pool: Arc<ClickHousePool>,
        table: impl Into<String>,
    ) -> AppResult<Self> {
        let kafka_consumer = KafkaEventConsumer::new(kafka_config)?;
        let table = table.into();
        let batch_config = BatchConfig::from_clickhouse_config(clickhouse_pool.config());
        let batch_writer = Arc::new(BatchWriter::new(
            clickhouse_pool.clone(),
            table.clone(),
            batch_config,
        ));

        Ok(Self {
            kafka_consumer,
            clickhouse_pool,
            batch_writer,
            table,
        })
    }

    /// 启动 CDC 同步
    pub async fn start(&self) -> AppResult<()> {
        info!(table = %self.table, "Starting CDC sync consumer");

        let batch_writer = self.batch_writer.clone();

        self.kafka_consumer
            .start(move |topic, payload| {
                let writer = batch_writer.clone();
                async move {
                    Self::handle_message(&writer, &topic, &payload).await
                }
            })
            .await
    }

    /// 处理单条消息
    async fn handle_message(
        batch_writer: &BatchWriter<CdcRecord>,
        _topic: &str,
        payload: &str,
    ) -> AppResult<()> {
        // 解析 Debezium 消息
        let msg: DebeziumMessage = serde_json::from_str(payload).map_err(|e| {
            AppError::validation(format!("Failed to parse CDC message: {}", e))
        })?;

        let record = CdcRecord::from_debezium(&msg);

        debug!(
            operation = %record.operation,
            table = %record.table_name,
            pk = %record.primary_key,
            "Processing CDC record"
        );

        batch_writer.insert(record).await?;

        Ok(())
    }

    /// 获取批量写入器
    pub fn batch_writer(&self) -> Arc<BatchWriter<CdcRecord>> {
        self.batch_writer.clone()
    }

    /// 刷新缓冲区
    pub async fn flush(&self) -> AppResult<()> {
        self.batch_writer.flush().await
    }
}

/// CDC 同步配置
#[derive(Debug, Clone)]
pub struct CdcSyncConfig {
    /// Kafka brokers
    pub brokers: String,
    /// Consumer group ID
    pub group_id: String,
    /// 要订阅的 topics
    pub topics: Vec<String>,
    /// ClickHouse 目标表
    pub target_table: String,
    /// 是否启用 DLQ
    pub enable_dlq: bool,
}

impl CdcSyncConfig {
    /// 创建新的配置
    pub fn new(
        brokers: impl Into<String>,
        group_id: impl Into<String>,
        target_table: impl Into<String>,
    ) -> Self {
        Self {
            brokers: brokers.into(),
            group_id: group_id.into(),
            topics: Vec::new(),
            target_table: target_table.into(),
            enable_dlq: true,
        }
    }

    /// 添加 topic
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topics.push(topic.into());
        self
    }

    /// 添加多个 topics
    pub fn with_topics(mut self, topics: Vec<String>) -> Self {
        self.topics.extend(topics);
        self
    }

    /// 设置 DLQ
    pub fn with_dlq(mut self, enable: bool) -> Self {
        self.enable_dlq = enable;
        self
    }

    /// 转换为 Kafka 消费者配置
    pub fn to_kafka_config(&self) -> KafkaConsumerConfig {
        KafkaConsumerConfig::new(&self.brokers, &self.group_id)
            .with_topics(self.topics.clone())
            .with_dlq_enabled(self.enable_dlq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdc_record_from_debezium() {
        let json = r#"{
            "payload": {
                "op": "c",
                "before": null,
                "after": {"id": 1, "name": "test"},
                "source": {
                    "table": "users",
                    "txId": 12345,
                    "db": "mydb"
                },
                "ts_ms": 1704067200000
            }
        }"#;

        let msg: DebeziumMessage = serde_json::from_str(json).unwrap();
        let record = CdcRecord::from_debezium(&msg);

        assert_eq!(record.operation, "INSERT");
        assert_eq!(record.table_name, "users");
        assert_eq!(record.transaction_id, Some("12345".to_string()));
    }

    #[test]
    fn test_cdc_sync_config() {
        let config = CdcSyncConfig::new("localhost:9092", "cdc-group", "cdc_records")
            .with_topic("dbserver.public.users")
            .with_topic("dbserver.public.orders")
            .with_dlq(true);

        assert_eq!(config.topics.len(), 2);
        assert!(config.enable_dlq);

        let kafka_config = config.to_kafka_config();
        assert_eq!(kafka_config.topics.len(), 2);
    }
}
