//! PostgreSQL Event Store 实现

use async_trait::async_trait;
use cuba_errors::{AppError, AppResult};
use cuba_event_core::{EventEnvelope, EventStore, StoredEvent};
use serde::Serialize;
use sqlx::PgPool;

/// PostgreSQL Event Store
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn append<E: Serialize + Send + Sync>(
        &self,
        envelope: &EventEnvelope<E>,
    ) -> AppResult<()> {
        let payload = serde_json::to_string(&envelope.data)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;

        let metadata = serde_json::to_string(&envelope.metadata)
            .map_err(|e| AppError::internal(format!("Failed to serialize metadata: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO event_store (id, aggregate_type, aggregate_id, event_type, version, payload, metadata, occurred_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(envelope.id)
        .bind(&envelope.aggregate_type)
        .bind(&envelope.aggregate_id)
        .bind(&envelope.event_type)
        .bind(envelope.version as i64)
        .bind(&payload)
        .bind(&metadata)
        .bind(envelope.occurred_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to append event: {}", e)))?;

        Ok(())
    }

    async fn get_events(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> AppResult<Vec<StoredEvent>> {
        let rows = sqlx::query_as::<_, StoredEventRow>(
            r#"
            SELECT id, aggregate_type, aggregate_id, event_type, version, payload, metadata, occurred_at
            FROM event_store
            WHERE aggregate_type = $1 AND aggregate_id = $2
            ORDER BY version ASC
            "#,
        )
        .bind(aggregate_type)
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to get events: {}", e)))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_events_from_version(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
        from_version: u64,
    ) -> AppResult<Vec<StoredEvent>> {
        let rows = sqlx::query_as::<_, StoredEventRow>(
            r#"
            SELECT id, aggregate_type, aggregate_id, event_type, version, payload, metadata, occurred_at
            FROM event_store
            WHERE aggregate_type = $1 AND aggregate_id = $2 AND version >= $3
            ORDER BY version ASC
            "#,
        )
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(from_version as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to get events: {}", e)))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_current_version(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> AppResult<u64> {
        let row: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT COALESCE(MAX(version), 0)
            FROM event_store
            WHERE aggregate_type = $1 AND aggregate_id = $2
            "#,
        )
        .bind(aggregate_type)
        .bind(aggregate_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to get version: {}", e)))?;

        Ok(row.map(|(v,)| v as u64).unwrap_or(0))
    }
}

#[derive(sqlx::FromRow)]
struct StoredEventRow {
    id: uuid::Uuid,
    aggregate_type: String,
    aggregate_id: String,
    event_type: String,
    version: i64,
    payload: String,
    metadata: String,
    occurred_at: chrono::DateTime<chrono::Utc>,
}

impl From<StoredEventRow> for StoredEvent {
    fn from(row: StoredEventRow) -> Self {
        Self {
            id: row.id.to_string(),
            aggregate_type: row.aggregate_type,
            aggregate_id: row.aggregate_id,
            event_type: row.event_type,
            version: row.version as u64,
            payload: row.payload,
            metadata: row.metadata,
            occurred_at: row.occurred_at,
        }
    }
}
