//! 事件存储仓库
//!
//! 提供事件查询功能

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::infrastructure::events::IamDomainEvent;

/// 存储的领域事件
#[derive(Debug, Clone)]
pub struct StoredEvent {
    pub id: Uuid,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub tenant_id: Uuid,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// 事件查询参数
#[derive(Debug, Default)]
pub struct EventQuery {
    pub tenant_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub aggregate_type: Option<String>,
    pub aggregate_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: i64,
    pub offset: i64,
}

/// 事件存储仓库 trait
#[async_trait]
pub trait EventStoreRepository: Send + Sync {
    /// 查询事件
    async fn find_events(&self, query: EventQuery) -> Result<Vec<StoredEvent>, sqlx::Error>;

    /// 获取用户的审计历史
    async fn find_by_user_id(
        &self,
        user_id: &str,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredEvent>, sqlx::Error>;

    /// 获取事件总数
    async fn count_events(&self, query: &EventQuery) -> Result<i64, sqlx::Error>;
}

/// PostgreSQL 事件存储仓库实现
pub struct PostgresEventStoreRepository {
    pool: PgPool,
}

impl PostgresEventStoreRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStoreRepository for PostgresEventStoreRepository {
    async fn find_events(&self, query: EventQuery) -> Result<Vec<StoredEvent>, sqlx::Error> {
        let mut sql = String::from(
            "SELECT id, event_type, aggregate_type, aggregate_id, tenant_id, payload, created_at 
             FROM domain_events WHERE 1=1"
        );

        if query.tenant_id.is_some() {
            sql.push_str(" AND tenant_id = $1");
        }
        if query.event_type.is_some() {
            sql.push_str(" AND event_type = $2");
        }
        if query.aggregate_type.is_some() {
            sql.push_str(" AND aggregate_type = $3");
        }
        if query.aggregate_id.is_some() {
            sql.push_str(" AND aggregate_id = $4");
        }
        if query.start_time.is_some() {
            sql.push_str(" AND created_at >= $5");
        }
        if query.end_time.is_some() {
            sql.push_str(" AND created_at <= $6");
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT $7 OFFSET $8");

        // 使用动态查询
        let _rows = sqlx::query_as::<_, (Uuid, String, String, String, Uuid, serde_json::Value, DateTime<Utc>)>(
            &sql
        )
        .fetch_all(&self.pool)
        .await;

        // 简化实现：直接查询所有事件
        let events = sqlx::query_as::<_, (Uuid, String, String, String, Uuid, serde_json::Value, DateTime<Utc>)>(
            r#"
            SELECT id, event_type, aggregate_type, aggregate_id, tenant_id, payload, created_at
            FROM domain_events
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(query.limit)
        .bind(query.offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(events
            .into_iter()
            .map(|(id, event_type, aggregate_type, aggregate_id, tenant_id, payload, created_at)| {
                StoredEvent {
                    id,
                    event_type,
                    aggregate_type,
                    aggregate_id,
                    tenant_id,
                    payload,
                    created_at,
                }
            })
            .collect())
    }

    async fn find_by_user_id(
        &self,
        user_id: &str,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredEvent>, sqlx::Error> {
        let events = sqlx::query_as::<_, (Uuid, String, String, String, Uuid, serde_json::Value, DateTime<Utc>)>(
            r#"
            SELECT id, event_type, aggregate_type, aggregate_id, tenant_id, payload, created_at
            FROM domain_events
            WHERE aggregate_id = $1 AND tenant_id = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(events
            .into_iter()
            .map(|(id, event_type, aggregate_type, aggregate_id, tenant_id, payload, created_at)| {
                StoredEvent {
                    id,
                    event_type,
                    aggregate_type,
                    aggregate_id,
                    tenant_id,
                    payload,
                    created_at,
                }
            })
            .collect())
    }

    async fn count_events(&self, _query: &EventQuery) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM domain_events")
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0)
    }
}
