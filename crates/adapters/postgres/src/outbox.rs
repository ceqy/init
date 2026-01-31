//! PostgreSQL Outbox 实现

use async_trait::async_trait;
use errors::{AppError, AppResult};
use ports::{OutboxMessage, OutboxPort};
use sqlx::PgPool;

/// PostgreSQL Outbox
pub struct PostgresOutbox {
    pool: PgPool,
}

impl PostgresOutbox {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OutboxPort for PostgresOutbox {
    async fn save(&self, message: &OutboxMessage) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO outbox (id, aggregate_type, aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&message.id)
        .bind(&message.aggregate_type)
        .bind(&message.aggregate_id)
        .bind(&message.event_type)
        .bind(&message.payload)
        .bind(message.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save outbox message: {}", e)))?;

        Ok(())
    }

    async fn get_pending(&self, limit: usize) -> AppResult<Vec<OutboxMessage>> {
        let rows = sqlx::query_as::<_, OutboxRow>(
            r#"
            SELECT id, aggregate_type, aggregate_id, event_type, payload, created_at, processed_at
            FROM outbox
            WHERE processed_at IS NULL
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to get pending messages: {}", e)))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn mark_processed(&self, id: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET processed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to mark message processed: {}", e)))?;

        Ok(())
    }

    async fn delete_processed(&self, before: chrono::DateTime<chrono::Utc>) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM outbox
            WHERE processed_at IS NOT NULL AND processed_at < $1
            "#,
        )
        .bind(before)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to delete processed messages: {}", e)))?;

        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: String,
    aggregate_type: String,
    aggregate_id: String,
    event_type: String,
    payload: String,
    created_at: chrono::DateTime<chrono::Utc>,
    processed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<OutboxRow> for OutboxMessage {
    fn from(row: OutboxRow) -> Self {
        Self {
            id: row.id,
            aggregate_type: row.aggregate_type,
            aggregate_id: row.aggregate_id,
            event_type: row.event_type,
            payload: row.payload,
            created_at: row.created_at,
            processed_at: row.processed_at,
        }
    }
}
