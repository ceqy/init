//! Outbox 仓储 - 用于可靠的事件发布
//!
//! Outbox 模式保证事件发布的最终一致性：
//! 1. 业务操作和事件写入在同一事务中
//! 2. 后台进程异步发布事件
//! 3. 发布成功后标记为已发布

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use errors::AppResult;
// use errors::AppError; // Removed unused
// use serde::Serialize; // Removed unused
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::error_mapper::map_sqlx_error;

/// Outbox 事件记录
#[derive(Debug, Clone)]
pub struct OutboxEvent {
    pub id: Uuid,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: String,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub retry_count: i32,
    pub last_error: Option<String>,
}

/// Outbox 仓储 trait
#[async_trait]
pub trait OutboxRepository: Send + Sync {
    /// 在事务中写入事件到 outbox
    async fn insert_in_tx<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        aggregate_type: &str,
        aggregate_id: Uuid,
        event_type: &str,
        payload_json: &str,
    ) -> AppResult<Uuid>;

    /// 获取待发布的事件 (按创建时间排序)
    async fn get_pending(&self, limit: i64) -> AppResult<Vec<OutboxEvent>>;

    /// 标记事件为已发布
    async fn mark_published(&self, id: Uuid) -> AppResult<()>;

    /// 标记事件发布失败并增加重试次数
    async fn mark_failed(&self, id: Uuid, error: &str) -> AppResult<()>;

    /// 删除已发布的事件 (可选的清理操作)
    async fn delete_published(&self, before: DateTime<Utc>) -> AppResult<u64>;
}

/// PostgreSQL Outbox 仓储实现
pub struct PostgresOutboxRepository {
    pool: PgPool,
}

impl PostgresOutboxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OutboxRepository for PostgresOutboxRepository {
    async fn insert_in_tx<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        aggregate_type: &str,
        aggregate_id: Uuid,
        event_type: &str,
        payload_json: &str,
    ) -> AppResult<Uuid> {
        let id = Uuid::now_v7();

        sqlx::query(
            r#"
            INSERT INTO outbox (id, aggregate_type, aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(event_type)
        .bind(serde_json::from_str::<serde_json::Value>(payload_json).unwrap_or_default())
        .bind(Utc::now())
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(id)
    }

    async fn get_pending(&self, limit: i64) -> AppResult<Vec<OutboxEvent>> {
        let rows = sqlx::query_as::<_, OutboxRow>(
            r#"
            SELECT id, aggregate_type, aggregate_id, event_type, payload, 
                   created_at, published_at, retry_count, last_error
            FROM outbox
            WHERE published_at IS NULL AND retry_count < 5
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn mark_published(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE outbox SET published_at = $1 WHERE id = $2")
            .bind(Utc::now())
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn mark_failed(&self, id: Uuid, error: &str) -> AppResult<()> {
        sqlx::query(
            "UPDATE outbox SET retry_count = retry_count + 1, last_error = $1 WHERE id = $2",
        )
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn delete_published(&self, before: DateTime<Utc>) -> AppResult<u64> {
        let result =
            sqlx::query("DELETE FROM outbox WHERE published_at IS NOT NULL AND published_at < $1")
                .bind(before)
                .execute(&self.pool)
                .await
                .map_err(map_sqlx_error)?;
        Ok(result.rows_affected())
    }
}

// ============ 数据行映射 ============

#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: Uuid,
    aggregate_type: String,
    aggregate_id: Uuid,
    event_type: String,
    payload: serde_json::Value,
    created_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
    retry_count: i32,
    last_error: Option<String>,
}

impl From<OutboxRow> for OutboxEvent {
    fn from(row: OutboxRow) -> Self {
        Self {
            id: row.id,
            aggregate_type: row.aggregate_type,
            aggregate_id: row.aggregate_id,
            event_type: row.event_type,
            payload: row.payload.to_string(),
            created_at: row.created_at,
            published_at: row.published_at,
            retry_count: row.retry_count,
            last_error: row.last_error,
        }
    }
}
