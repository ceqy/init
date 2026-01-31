//! Outbox 同步模块
//!
//! 从 PostgreSQL outbox 表读取事件并同步到 ClickHouse

use std::sync::Arc;
use std::time::Duration;

use clickhouse::Row;
use errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::{debug, error, info};

use crate::batch::BatchWriter;
use crate::client::ClickHousePool;
use crate::config::BatchConfig;

/// PostgreSQL Outbox 消息（用于从数据库读取）
#[derive(Debug, Clone, FromRow)]
pub struct OutboxMessageRow {
    pub id: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Outbox 事件记录（ClickHouse 格式）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct OutboxEventRecord {
    /// 事件 ID
    pub id: String,
    /// 聚合类型
    pub aggregate_type: String,
    /// 聚合 ID
    pub aggregate_id: String,
    /// 事件类型
    pub event_type: String,
    /// 负载
    pub payload: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 处理时间
    pub processed_at: chrono::DateTime<chrono::Utc>,
}

impl From<OutboxMessageRow> for OutboxEventRecord {
    fn from(msg: OutboxMessageRow) -> Self {
        Self {
            id: msg.id,
            aggregate_type: msg.aggregate_type,
            aggregate_id: msg.aggregate_id,
            event_type: msg.event_type,
            payload: msg.payload,
            created_at: msg.created_at,
            processed_at: msg.processed_at.unwrap_or_else(chrono::Utc::now),
        }
    }
}

/// Outbox 同步配置
#[derive(Debug, Clone)]
pub struct OutboxSyncConfig {
    /// 批量大小
    pub batch_size: usize,
    /// 轮询间隔
    pub poll_interval: Duration,
    /// PostgreSQL outbox 表名
    pub source_table: String,
    /// ClickHouse 目标表名
    pub target_table: String,
    /// 是否在同步后删除源记录
    pub delete_after_sync: bool,
    /// 保留已处理记录的时间
    pub retention_period: Duration,
}

impl Default for OutboxSyncConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            poll_interval: Duration::from_secs(5),
            source_table: "outbox".to_string(),
            target_table: "outbox_events".to_string(),
            delete_after_sync: false,
            retention_period: Duration::from_secs(7 * 24 * 3600), // 7 days
        }
    }
}

impl OutboxSyncConfig {
    /// 创建新的配置
    pub fn new(source_table: impl Into<String>, target_table: impl Into<String>) -> Self {
        Self {
            source_table: source_table.into(),
            target_table: target_table.into(),
            ..Default::default()
        }
    }

    /// 设置批量大小
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// 设置轮询间隔
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// 设置同步后删除
    pub fn with_delete_after_sync(mut self, delete: bool) -> Self {
        self.delete_after_sync = delete;
        self
    }
}

/// Outbox 同步处理器
pub struct OutboxSyncProcessor {
    pg_pool: PgPool,
    #[allow(dead_code)]
    clickhouse_pool: Arc<ClickHousePool>,
    batch_writer: Arc<BatchWriter<OutboxEventRecord>>,
    config: OutboxSyncConfig,
}

impl OutboxSyncProcessor {
    /// 创建新的 Outbox 同步处理器
    pub fn new(
        pg_pool: PgPool,
        clickhouse_pool: Arc<ClickHousePool>,
        config: OutboxSyncConfig,
    ) -> Self {
        let batch_config = BatchConfig::new(config.batch_size, config.poll_interval);
        let batch_writer = Arc::new(BatchWriter::new(
            clickhouse_pool.clone(),
            config.target_table.clone(),
            batch_config,
        ));

        Self {
            pg_pool,
            clickhouse_pool,
            batch_writer,
            config,
        }
    }

    /// 启动同步
    pub async fn start(&self) -> AppResult<()> {
        info!(
            source = %self.config.source_table,
            target = %self.config.target_table,
            batch_size = self.config.batch_size,
            "Starting outbox sync processor"
        );

        loop {
            match self.sync_batch().await {
                Ok(count) => {
                    if count > 0 {
                        debug!(count, "Synced outbox records");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to sync outbox batch");
                }
            }

            tokio::time::sleep(self.config.poll_interval).await;
        }
    }

    /// 同步一批数据
    async fn sync_batch(&self) -> AppResult<usize> {
        // 从 PostgreSQL 读取未处理的 outbox 消息
        let messages = self.fetch_pending_messages().await?;

        if messages.is_empty() {
            return Ok(0);
        }

        let count = messages.len();
        let ids: Vec<String> = messages.iter().map(|m| m.id.clone()).collect();

        // 转换为 ClickHouse 记录
        let records: Vec<OutboxEventRecord> = messages
            .into_iter()
            .map(OutboxEventRecord::from)
            .collect();

        // 写入 ClickHouse
        self.batch_writer.insert_many(records).await?;
        self.batch_writer.flush().await?;

        // 标记为已处理
        self.mark_processed(&ids).await?;

        // 可选：删除已处理的记录
        if self.config.delete_after_sync {
            self.delete_processed(&ids).await?;
        }

        Ok(count)
    }

    /// 获取待处理的消息
    async fn fetch_pending_messages(&self) -> AppResult<Vec<OutboxMessageRow>> {
        let query = format!(
            r#"
            SELECT id, aggregate_type, aggregate_id, event_type, payload, created_at, processed_at
            FROM {}
            WHERE processed_at IS NULL
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
            self.config.source_table
        );

        let messages: Vec<OutboxMessageRow> = sqlx::query_as(&query)
            .bind(self.config.batch_size as i64)
            .fetch_all(&self.pg_pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to fetch outbox messages: {}", e)))?;

        Ok(messages)
    }

    /// 标记消息为已处理
    async fn mark_processed(&self, ids: &[String]) -> AppResult<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${}", i)).collect();
        let query = format!(
            "UPDATE {} SET processed_at = NOW() WHERE id IN ({})",
            self.config.source_table,
            placeholders.join(", ")
        );

        let mut query_builder = sqlx::query(&query);
        for id in ids {
            query_builder = query_builder.bind(id);
        }

        query_builder
            .execute(&self.pg_pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to mark messages as processed: {}", e)))?;

        Ok(())
    }

    /// 删除已处理的消息
    async fn delete_processed(&self, ids: &[String]) -> AppResult<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${}", i)).collect();
        let query = format!(
            "DELETE FROM {} WHERE id IN ({})",
            self.config.source_table,
            placeholders.join(", ")
        );

        let mut query_builder = sqlx::query(&query);
        for id in ids {
            query_builder = query_builder.bind(id);
        }

        query_builder
            .execute(&self.pg_pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete processed messages: {}", e)))?;

        Ok(())
    }

    /// 清理过期的已处理记录
    pub async fn cleanup_old_records(&self) -> AppResult<u64> {
        let retention_secs = self.config.retention_period.as_secs() as i64;

        let query = format!(
            r#"
            DELETE FROM {}
            WHERE processed_at IS NOT NULL
            AND processed_at < NOW() - INTERVAL '{} seconds'
            "#,
            self.config.source_table, retention_secs
        );

        let result = sqlx::query(&query)
            .execute(&self.pg_pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to cleanup old records: {}", e)))?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            info!(deleted, "Cleaned up old outbox records");
        }

        Ok(deleted)
    }

    /// 获取批量写入器
    pub fn batch_writer(&self) -> Arc<BatchWriter<OutboxEventRecord>> {
        self.batch_writer.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outbox_sync_config() {
        let config = OutboxSyncConfig::new("my_outbox", "ch_outbox_events")
            .with_batch_size(500)
            .with_poll_interval(Duration::from_secs(10))
            .with_delete_after_sync(true);

        assert_eq!(config.source_table, "my_outbox");
        assert_eq!(config.target_table, "ch_outbox_events");
        assert_eq!(config.batch_size, 500);
        assert!(config.delete_after_sync);
    }

    #[test]
    fn test_outbox_event_record_from_message() {
        let msg = OutboxMessageRow {
            id: "test-id".to_string(),
            aggregate_type: "Order".to_string(),
            aggregate_id: "order-123".to_string(),
            event_type: "OrderCreated".to_string(),
            payload: r#"{"amount": 100}"#.to_string(),
            created_at: chrono::Utc::now(),
            processed_at: Some(chrono::Utc::now()),
        };

        let record = OutboxEventRecord::from(msg);
        assert_eq!(record.id, "test-id");
        assert_eq!(record.aggregate_type, "Order");
        assert_eq!(record.event_type, "OrderCreated");
    }
}
