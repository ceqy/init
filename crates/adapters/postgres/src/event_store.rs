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

        // 使用事务确保版本检查和插入的原子性
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        // 检查当前版本（乐观并发控制）
        // 注意：不能在聚合函数上使用 FOR UPDATE，所以我们依赖唯一约束来防止冲突
        let current_version: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT COALESCE(MAX(version), 0)
            FROM event_store
            WHERE aggregate_type = $1 AND aggregate_id = $2
            "#,
        )
        .bind(&envelope.aggregate_type)
        .bind(&envelope.aggregate_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::database(format!("Failed to get current version: {}", e)))?;

        let current_version = current_version.map(|(v,)| v as u64).unwrap_or(0);
        let expected_version = if envelope.version == 1 {
            0
        } else {
            envelope.version - 1
        };

        // 版本冲突检测
        if current_version != expected_version {
            return Err(AppError::conflict(format!(
                "Version conflict for {}:{} - expected version {}, but current version is {}",
                envelope.aggregate_type, envelope.aggregate_id, expected_version, current_version
            )));
        }

        // 插入事件
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
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            // 检查是否是唯一约束冲突（版本冲突）
            if let Some(db_err) = e.as_database_error()
                && db_err.constraint() == Some("uk_event_store_aggregate_version") {
                    return AppError::conflict(format!(
                        "Version conflict for {}:{} - version {} already exists",
                        envelope.aggregate_type, envelope.aggregate_id, envelope.version
                    ));
                }
            AppError::database(format!("Failed to append event: {}", e))
        })?;

        // 提交事务
        tx.commit()
            .await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use cuba_event_core::EventMetadata;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        message: String,
    }

    fn create_test_envelope(
        aggregate_id: &str,
        version: u64,
        message: &str,
    ) -> EventEnvelope<TestEvent> {
        EventEnvelope {
            id: Uuid::new_v4(),
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: aggregate_id.to_string(),
            event_type: "TestEvent".to_string(),
            version,
            data: TestEvent {
                message: message.to_string(),
            },
            metadata: EventMetadata::new(),
            occurred_at: chrono::Utc::now(),
        }
    }

    async fn setup_event_store_table(pool: &PgPool) -> AppResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS event_store (
                id UUID PRIMARY KEY,
                aggregate_type VARCHAR(50) NOT NULL,
                aggregate_id VARCHAR(100) NOT NULL,
                event_type VARCHAR(100) NOT NULL,
                version BIGINT NOT NULL,
                payload TEXT NOT NULL,
                metadata TEXT NOT NULL,
                occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                CONSTRAINT uk_event_store_aggregate_version UNIQUE (aggregate_type, aggregate_id, version)
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to create event_store table: {}", e)))?;

        Ok(())
    }

    #[sqlx::test]
    async fn test_append_first_event_success(pool: PgPool) -> AppResult<()> {
        setup_event_store_table(&pool).await?;
        let store = PostgresEventStore::new(pool);
        let aggregate_id = Uuid::new_v4().to_string();

        // 追加第一个事件（版本 1）
        let envelope = create_test_envelope(&aggregate_id, 1, "First event");
        store.append(&envelope).await?;

        // 验证版本
        let version = store
            .get_current_version("TestAggregate", &aggregate_id)
            .await?;
        assert_eq!(version, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_append_sequential_events_success(pool: PgPool) -> AppResult<()> {
        setup_event_store_table(&pool).await?;
        let store = PostgresEventStore::new(pool);
        let aggregate_id = Uuid::new_v4().to_string();

        // 追加多个事件
        for i in 1..=5 {
            let envelope = create_test_envelope(&aggregate_id, i, &format!("Event {}", i));
            store.append(&envelope).await?;
        }

        // 验证最终版本
        let version = store
            .get_current_version("TestAggregate", &aggregate_id)
            .await?;
        assert_eq!(version, 5);

        // 验证所有事件都被保存
        let events = store.get_events("TestAggregate", &aggregate_id).await?;
        assert_eq!(events.len(), 5);

        Ok(())
    }

    #[sqlx::test]
    async fn test_version_conflict_detection(pool: PgPool) -> AppResult<()> {
        setup_event_store_table(&pool).await?;
        let store = PostgresEventStore::new(pool);
        let aggregate_id = Uuid::new_v4().to_string();

        // 追加第一个事件
        let envelope1 = create_test_envelope(&aggregate_id, 1, "Event 1");
        store.append(&envelope1).await?;

        // 尝试追加版本 1 的另一个事件（应该失败）
        let envelope2 = create_test_envelope(&aggregate_id, 1, "Conflicting Event 1");
        let result = store.append(&envelope2).await;

        // 验证返回冲突错误
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));

        // 验证版本仍然是 1
        let version = store
            .get_current_version("TestAggregate", &aggregate_id)
            .await?;
        assert_eq!(version, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_skip_version_detection(pool: PgPool) -> AppResult<()> {
        setup_event_store_table(&pool).await?;
        let store = PostgresEventStore::new(pool);
        let aggregate_id = Uuid::new_v4().to_string();

        // 追加第一个事件
        let envelope1 = create_test_envelope(&aggregate_id, 1, "Event 1");
        store.append(&envelope1).await?;

        // 尝试跳过版本 2，直接追加版本 3（应该失败）
        let envelope3 = create_test_envelope(&aggregate_id, 3, "Event 3");
        let result = store.append(&envelope3).await;

        // 验证返回冲突错误
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));

        // 验证版本仍然是 1
        let version = store
            .get_current_version("TestAggregate", &aggregate_id)
            .await?;
        assert_eq!(version, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_events_from_version(pool: PgPool) -> AppResult<()> {
        setup_event_store_table(&pool).await?;
        let store = PostgresEventStore::new(pool);
        let aggregate_id = Uuid::new_v4().to_string();

        // 追加 5 个事件
        for i in 1..=5 {
            let envelope = create_test_envelope(&aggregate_id, i, &format!("Event {}", i));
            store.append(&envelope).await?;
        }

        // 获取从版本 3 开始的事件
        let events = store
            .get_events_from_version("TestAggregate", &aggregate_id, 3)
            .await?;

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].version, 3);
        assert_eq!(events[1].version, 4);
        assert_eq!(events[2].version, 5);

        Ok(())
    }

    #[sqlx::test]
    async fn test_different_aggregates_independent_versions(pool: PgPool) -> AppResult<()> {
        setup_event_store_table(&pool).await?;
        let store = PostgresEventStore::new(pool);
        let aggregate_id1 = Uuid::new_v4().to_string();
        let aggregate_id2 = Uuid::new_v4().to_string();

        // 为两个不同的聚合追加事件
        let envelope1 = create_test_envelope(&aggregate_id1, 1, "Aggregate 1 Event 1");
        store.append(&envelope1).await?;

        let envelope2 = create_test_envelope(&aggregate_id2, 1, "Aggregate 2 Event 1");
        store.append(&envelope2).await?;

        // 验证两个聚合的版本都是 1
        let version1 = store
            .get_current_version("TestAggregate", &aggregate_id1)
            .await?;
        let version2 = store
            .get_current_version("TestAggregate", &aggregate_id2)
            .await?;
        assert_eq!(version1, 1);
        assert_eq!(version2, 1);

        Ok(())
    }
}
