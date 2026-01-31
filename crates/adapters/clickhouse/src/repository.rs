//! ClickHouse Repository 实现
//!
//! 实现分析 Repository trait

use std::sync::Arc;

use async_trait::async_trait;
use clickhouse::Row;
use cuba_errors::{AppError, AppResult};
use cuba_ports::{
    AnalyticsQueryOptions, AuditLogEntry, AuditLogFilter, AuditLogRepository,
    BusinessEvent, BusinessEventRepository, EventTrend,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::batch::BatchWriter;
use crate::client::ClickHousePool;
use crate::config::BatchConfig;
use crate::models::{AuditLogRow, BusinessEventRow};

/// 审计日志 Repository 实现
pub struct ClickHouseAuditLogRepository {
    pool: Arc<ClickHousePool>,
    batch_writer: Arc<BatchWriter<AuditLogRow>>,
    table: String,
}

impl ClickHouseAuditLogRepository {
    /// 创建新的审计日志 Repository
    pub fn new(pool: Arc<ClickHousePool>, table: impl Into<String>) -> Self {
        let table = table.into();
        let batch_config = BatchConfig::from_clickhouse_config(pool.config());
        let batch_writer = Arc::new(BatchWriter::new(
            pool.clone(),
            table.clone(),
            batch_config,
        ));

        Self {
            pool,
            batch_writer,
            table,
        }
    }

    /// 获取批量写入器
    pub fn batch_writer(&self) -> Arc<BatchWriter<AuditLogRow>> {
        self.batch_writer.clone()
    }

    /// 启动后台刷新
    pub fn start_background_flush(&self) -> tokio::task::JoinHandle<()> {
        self.batch_writer.clone().start_background_flush()
    }

    /// 构建查询条件
    fn build_where_clause(
        &self,
        filter: &AuditLogFilter,
        options: &AnalyticsQueryOptions,
    ) -> String {
        let mut conditions = Vec::new();

        if let Some(tenant_id) = options.tenant_id {
            conditions.push(format!("tenant_id = '{}'", tenant_id));
        }

        if let Some(start) = options.start_time {
            conditions.push(format!("timestamp >= '{}'", start.format("%Y-%m-%d %H:%M:%S")));
        }

        if let Some(end) = options.end_time {
            conditions.push(format!("timestamp <= '{}'", end.format("%Y-%m-%d %H:%M:%S")));
        }

        if let Some(user_id) = filter.user_id {
            conditions.push(format!("user_id = '{}'", user_id));
        }

        if let Some(action) = &filter.action {
            conditions.push(format!("action = '{}'", action));
        }

        if let Some(resource_type) = &filter.resource_type {
            conditions.push(format!("resource_type = '{}'", resource_type));
        }

        if let Some(resource_id) = &filter.resource_id {
            conditions.push(format!("resource_id = '{}'", resource_id));
        }

        if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        }
    }
}

#[async_trait]
impl AuditLogRepository for ClickHouseAuditLogRepository {
    async fn log(&self, entry: AuditLogEntry) -> AppResult<()> {
        let row = AuditLogRow::from(entry);
        self.batch_writer.insert(row).await
    }

    async fn log_batch(&self, entries: Vec<AuditLogEntry>) -> AppResult<u64> {
        let count = entries.len() as u64;
        let rows: Vec<AuditLogRow> = entries.into_iter().map(AuditLogRow::from).collect();
        self.batch_writer.insert_many(rows).await?;
        Ok(count)
    }

    async fn query(
        &self,
        filter: AuditLogFilter,
        options: &AnalyticsQueryOptions,
    ) -> AppResult<Vec<AuditLogEntry>> {
        let where_clause = self.build_where_clause(&filter, options);

        let order_by = options
            .order_by
            .as_deref()
            .unwrap_or("timestamp");
        let order_dir = if options.descending { "DESC" } else { "ASC" };

        let query = format!(
            "SELECT * FROM {} {} ORDER BY {} {} LIMIT {} OFFSET {}",
            self.table, where_clause, order_by, order_dir, options.limit, options.offset
        );

        debug!(query = %query, "Executing audit log query");

        let client = self.pool.get().await?;
        let rows: Vec<AuditLogRow> = client
            .query(&query)
            .fetch_all()
            .await
            .map_err(|e| AppError::database(format!("Failed to query audit logs: {}", e)))?;

        Ok(rows.into_iter().map(AuditLogEntry::from).collect())
    }

    async fn count(
        &self,
        filter: AuditLogFilter,
        options: &AnalyticsQueryOptions,
    ) -> AppResult<u64> {
        let where_clause = self.build_where_clause(&filter, options);

        let query = format!(
            "SELECT count() FROM {} {}",
            self.table, where_clause
        );

        let client = self.pool.get().await?;
        let count: u64 = client
            .query(&query)
            .fetch_one()
            .await
            .map_err(|e| AppError::database(format!("Failed to count audit logs: {}", e)))?;

        Ok(count)
    }
}

/// 业务事件 Repository 实现
pub struct ClickHouseBusinessEventRepository {
    pool: Arc<ClickHousePool>,
    batch_writer: Arc<BatchWriter<BusinessEventRow>>,
    table: String,
}

impl ClickHouseBusinessEventRepository {
    /// 创建新的业务事件 Repository
    pub fn new(pool: Arc<ClickHousePool>, table: impl Into<String>) -> Self {
        let table = table.into();
        let batch_config = BatchConfig::from_clickhouse_config(pool.config());
        let batch_writer = Arc::new(BatchWriter::new(
            pool.clone(),
            table.clone(),
            batch_config,
        ));

        Self {
            pool,
            batch_writer,
            table,
        }
    }

    /// 获取批量写入器
    pub fn batch_writer(&self) -> Arc<BatchWriter<BusinessEventRow>> {
        self.batch_writer.clone()
    }

    /// 启动后台刷新
    pub fn start_background_flush(&self) -> tokio::task::JoinHandle<()> {
        self.batch_writer.clone().start_background_flush()
    }

    /// 构建查询条件
    fn build_where_clause(&self, options: &AnalyticsQueryOptions) -> String {
        let mut conditions = Vec::new();

        if let Some(tenant_id) = options.tenant_id {
            conditions.push(format!("tenant_id = '{}'", tenant_id));
        }

        if let Some(start) = options.start_time {
            conditions.push(format!("timestamp >= '{}'", start.format("%Y-%m-%d %H:%M:%S")));
        }

        if let Some(end) = options.end_time {
            conditions.push(format!("timestamp <= '{}'", end.format("%Y-%m-%d %H:%M:%S")));
        }

        if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        }
    }
}

#[async_trait]
impl BusinessEventRepository for ClickHouseBusinessEventRepository {
    async fn record(&self, event: BusinessEvent) -> AppResult<()> {
        let row = BusinessEventRow::from(event);
        self.batch_writer.insert(row).await
    }

    async fn record_batch(&self, events: Vec<BusinessEvent>) -> AppResult<u64> {
        let count = events.len() as u64;
        let rows: Vec<BusinessEventRow> = events.into_iter().map(BusinessEventRow::from).collect();
        self.batch_writer.insert_many(rows).await?;
        Ok(count)
    }

    async fn query(&self, options: &AnalyticsQueryOptions) -> AppResult<Vec<BusinessEvent>> {
        let where_clause = self.build_where_clause(options);

        let order_by = options
            .order_by
            .as_deref()
            .unwrap_or("timestamp");
        let order_dir = if options.descending { "DESC" } else { "ASC" };

        let query = format!(
            "SELECT * FROM {} {} ORDER BY {} {} LIMIT {} OFFSET {}",
            self.table, where_clause, order_by, order_dir, options.limit, options.offset
        );

        debug!(query = %query, "Executing business event query");

        let client = self.pool.get().await?;
        let rows: Vec<BusinessEventRow> = client
            .query(&query)
            .fetch_all()
            .await
            .map_err(|e| AppError::database(format!("Failed to query business events: {}", e)))?;

        Ok(rows.into_iter().map(BusinessEvent::from).collect())
    }

    async fn query_by_aggregate(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
        options: &AnalyticsQueryOptions,
    ) -> AppResult<Vec<BusinessEvent>> {
        let mut where_clause = self.build_where_clause(options);

        let aggregate_condition = format!(
            "aggregate_type = '{}' AND aggregate_id = '{}'",
            aggregate_type, aggregate_id
        );

        if where_clause.is_empty() {
            where_clause = format!("WHERE {}", aggregate_condition);
        } else {
            where_clause = format!("{} AND {}", where_clause, aggregate_condition);
        }

        let query = format!(
            "SELECT * FROM {} {} ORDER BY version ASC LIMIT {} OFFSET {}",
            self.table, where_clause, options.limit, options.offset
        );

        let client = self.pool.get().await?;
        let rows: Vec<BusinessEventRow> = client
            .query(&query)
            .fetch_all()
            .await
            .map_err(|e| AppError::database(format!("Failed to query events by aggregate: {}", e)))?;

        Ok(rows.into_iter().map(BusinessEvent::from).collect())
    }

    async fn trend_analysis(
        &self,
        event_type: &str,
        options: &AnalyticsQueryOptions,
    ) -> AppResult<Vec<EventTrend>> {
        let mut conditions = vec![format!("event_type = '{}'", event_type)];

        if let Some(tenant_id) = options.tenant_id {
            conditions.push(format!("tenant_id = '{}'", tenant_id));
        }

        if let Some(start) = options.start_time {
            conditions.push(format!("timestamp >= '{}'", start.format("%Y-%m-%d %H:%M:%S")));
        }

        if let Some(end) = options.end_time {
            conditions.push(format!("timestamp <= '{}'", end.format("%Y-%m-%d %H:%M:%S")));
        }

        let where_clause = format!("WHERE {}", conditions.join(" AND "));

        let query = format!(
            r#"
            SELECT
                toStartOfHour(timestamp) as time_bucket,
                event_type,
                count() as count
            FROM {}
            {}
            GROUP BY time_bucket, event_type
            ORDER BY time_bucket ASC
            LIMIT {}
            "#,
            self.table, where_clause, options.limit
        );

        let client = self.pool.get().await?;
        let trends: Vec<EventTrendRow> = client
            .query(&query)
            .fetch_all()
            .await
            .map_err(|e| AppError::database(format!("Failed to analyze trends: {}", e)))?;

        Ok(trends.into_iter().map(EventTrend::from).collect())
    }
}

/// 事件趋势行（用于查询结果）
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
struct EventTrendRow {
    time_bucket: chrono::DateTime<chrono::Utc>,
    event_type: String,
    count: u64,
}

impl From<EventTrendRow> for EventTrend {
    fn from(row: EventTrendRow) -> Self {
        Self {
            time_bucket: row.time_bucket,
            event_type: row.event_type,
            count: row.count,
        }
    }
}
