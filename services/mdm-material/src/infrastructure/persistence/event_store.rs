//! 事件存储实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::types::{Pagination, TenantId};
use errors::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::events::MaterialEvent;
use crate::domain::value_objects::MaterialId;

/// 事件存储接口
#[async_trait]
pub trait EventStore: Send + Sync {
    /// 保存事件
    async fn save_event(&self, event: &MaterialEvent) -> AppResult<()>;

    /// 获取物料的所有事件
    async fn get_events(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<MaterialEvent>>;

    /// 按时间范围获取事件
    async fn get_events_by_time_range(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        pagination: Pagination,
    ) -> AppResult<(Vec<MaterialEvent>, i64)>;
}

/// PostgreSQL 事件存储实现
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
    async fn save_event(&self, event: &MaterialEvent) -> AppResult<()> {
        let (event_type, event_id, material_id, tenant_id, user_id, occurred_at) =
            extract_event_metadata(event);

        let event_data = serde_json::to_value(event)
            .map_err(|e| AppError::internal(format!("序列化事件失败: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO material_events (
                event_id, material_id, tenant_id, event_type, event_data,
                occurred_at, user_id, aggregate_version, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7,
                (SELECT COALESCE(MAX(aggregate_version), 0) + 1
                 FROM material_events
                 WHERE material_id = $2),
                NOW())
            "#,
        )
        .bind(event_id)
        .bind(material_id)
        .bind(tenant_id)
        .bind(event_type)
        .bind(event_data)
        .bind(occurred_at)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("保存事件失败: {}", e)))?;

        Ok(())
    }

    async fn get_events(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<MaterialEvent>> {
        let rows: Vec<EventRow> = sqlx::query_as(
            r#"
            SELECT event_id, material_id, tenant_id, event_type, event_data,
                   occurred_at, user_id, aggregate_version
            FROM material_events
            WHERE material_id = $1 AND tenant_id = $2
            ORDER BY aggregate_version ASC
            "#,
        )
        .bind(material_id.0)
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询事件失败: {}", e)))?;

        rows.into_iter()
            .filter_map(|row| deserialize_event(&row.event_data))
            .collect::<Result<Vec<_>, _>>()
    }

    async fn get_events_by_time_range(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        pagination: Pagination,
    ) -> AppResult<(Vec<MaterialEvent>, i64)> {
        // 构建查询条件
        let mut query = String::from(
            r#"
            SELECT event_id, material_id, tenant_id, event_type, event_data,
                   occurred_at, user_id, aggregate_version
            FROM material_events
            WHERE material_id = $1 AND tenant_id = $2
            "#,
        );

        let mut bind_index = 3;
        if from.is_some() {
            query.push_str(&format!(" AND occurred_at >= ${}", bind_index));
            bind_index += 1;
        }
        if to.is_some() {
            query.push_str(&format!(" AND occurred_at <= ${}", bind_index));
            bind_index += 1;
        }

        query.push_str(" ORDER BY occurred_at DESC");
        query.push_str(&format!(" LIMIT ${} OFFSET ${}", bind_index, bind_index + 1));

        // 执行查询
        let offset = (pagination.page - 1) * pagination.page_size;
        let mut query_builder = sqlx::query_as::<_, EventRow>(&query)
            .bind(material_id.0)
            .bind(tenant_id.0);

        if let Some(from_date) = from {
            query_builder = query_builder.bind(from_date);
        }
        if let Some(to_date) = to {
            query_builder = query_builder.bind(to_date);
        }

        let rows = query_builder
            .bind(pagination.page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("查询事件失败: {}", e)))?;

        // 查询总数
        let mut count_query = String::from(
            "SELECT COUNT(*) FROM material_events WHERE material_id = $1 AND tenant_id = $2",
        );

        let mut count_bind_index = 3;
        if from.is_some() {
            count_query.push_str(&format!(" AND occurred_at >= ${}", count_bind_index));
            count_bind_index += 1;
        }
        if to.is_some() {
            count_query.push_str(&format!(" AND occurred_at <= ${}", count_bind_index));
        }

        let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query)
            .bind(material_id.0)
            .bind(tenant_id.0);

        if let Some(from_date) = from {
            count_query_builder = count_query_builder.bind(from_date);
        }
        if let Some(to_date) = to {
            count_query_builder = count_query_builder.bind(to_date);
        }

        let total = count_query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("查询事件总数失败: {}", e)))?;

        // 反序列化事件
        let events = rows
            .into_iter()
            .filter_map(|row| deserialize_event(&row.event_data))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((events, total))
    }
}

/// 事件行结构
#[derive(sqlx::FromRow)]
struct EventRow {
    #[allow(dead_code)]
    event_id: Uuid,
    #[allow(dead_code)]
    material_id: Uuid,
    #[allow(dead_code)]
    tenant_id: Uuid,
    #[allow(dead_code)]
    event_type: String,
    event_data: serde_json::Value,
    #[allow(dead_code)]
    occurred_at: DateTime<Utc>,
    #[allow(dead_code)]
    user_id: Option<String>,
    #[allow(dead_code)]
    aggregate_version: i32,
}

/// 从事件中提取元数据
fn extract_event_metadata(
    event: &MaterialEvent,
) -> (String, Uuid, Uuid, Uuid, Option<String>, DateTime<Utc>) {
    match event {
        MaterialEvent::Created(e) => (
            "Created".to_string(),
            e.metadata.event_id,
            e.material_id.0,
            e.metadata.tenant_id.0,
            e.metadata.user_id.clone(),
            e.metadata.occurred_at,
        ),
        MaterialEvent::Updated(e) => (
            "Updated".to_string(),
            e.metadata.event_id,
            e.material_id.0,
            e.metadata.tenant_id.0,
            e.metadata.user_id.clone(),
            e.metadata.occurred_at,
        ),
        MaterialEvent::Activated(e) => (
            "Activated".to_string(),
            e.metadata.event_id,
            e.material_id.0,
            e.metadata.tenant_id.0,
            e.metadata.user_id.clone(),
            e.metadata.occurred_at,
        ),
        MaterialEvent::Deactivated(e) => (
            "Deactivated".to_string(),
            e.metadata.event_id,
            e.material_id.0,
            e.metadata.tenant_id.0,
            e.metadata.user_id.clone(),
            e.metadata.occurred_at,
        ),
        MaterialEvent::Blocked(e) => (
            "Blocked".to_string(),
            e.metadata.event_id,
            e.material_id.0,
            e.metadata.tenant_id.0,
            e.metadata.user_id.clone(),
            e.metadata.occurred_at,
        ),
        MaterialEvent::MarkedForDeletion(e) => (
            "MarkedForDeletion".to_string(),
            e.metadata.event_id,
            e.material_id.0,
            e.metadata.tenant_id.0,
            e.metadata.user_id.clone(),
            e.metadata.occurred_at,
        ),
        MaterialEvent::Deleted(e) => (
            "Deleted".to_string(),
            e.metadata.event_id,
            e.material_id.0,
            e.metadata.tenant_id.0,
            e.metadata.user_id.clone(),
            e.metadata.occurred_at,
        ),
        MaterialEvent::ExtendedToPlant(e) => (
            "ExtendedToPlant".to_string(),
            e.metadata.event_id,
            e.material_id.0,
            e.metadata.tenant_id.0,
            e.metadata.user_id.clone(),
            e.metadata.occurred_at,
        ),
        MaterialEvent::ExtendedToSalesOrg(e) => (
            "ExtendedToSalesOrg".to_string(),
            e.metadata.event_id,
            e.material_id.0,
            e.metadata.tenant_id.0,
            e.metadata.user_id.clone(),
            e.metadata.occurred_at,
        ),
    }
}

/// 反序列化事件
fn deserialize_event(event_data: &serde_json::Value) -> Option<AppResult<MaterialEvent>> {
    Some(
        serde_json::from_value(event_data.clone())
            .map_err(|e| AppError::internal(format!("反序列化事件失败: {}", e))),
    )
}
