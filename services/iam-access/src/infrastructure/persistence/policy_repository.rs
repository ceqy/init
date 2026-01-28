//! PostgreSQL 策略仓储实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cuba_common::{AuditInfo, TenantId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::policy::{Effect, Policy, PolicyId, PolicyRepository};

/// 将 sqlx 错误转换为 AppError
fn map_sqlx_error(e: sqlx::Error) -> AppError {
    AppError::database(e.to_string())
}

/// 将 serde_json 错误转换为 AppError
fn map_json_error(e: serde_json::Error) -> AppError {
    AppError::internal(format!("JSON serialization error: {}", e))
}

pub struct PostgresPolicyRepository {
    pool: PgPool,
}

impl PostgresPolicyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PolicyRepository for PostgresPolicyRepository {
    async fn create(&self, policy: &Policy) -> AppResult<()> {
        let subjects_json = serde_json::to_value(&policy.subjects).map_err(map_json_error)?;
        let resources_json = serde_json::to_value(&policy.resources).map_err(map_json_error)?;
        let actions_json = serde_json::to_value(&policy.actions).map_err(map_json_error)?;
        let conditions_json: Option<serde_json::Value> = policy
            .conditions
            .as_ref()
            .map(|c| serde_json::from_str(c))
            .transpose()
            .map_err(map_json_error)?;

        sqlx::query(
            r#"
            INSERT INTO policies (id, tenant_id, name, description, effect, subjects, resources, actions, conditions, priority, is_active, created_at, created_by, updated_at, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(policy.id.0)
        .bind(policy.tenant_id.0)
        .bind(&policy.name)
        .bind(&policy.description)
        .bind(policy.effect.to_string())
        .bind(&subjects_json)
        .bind(&resources_json)
        .bind(&actions_json)
        .bind(&conditions_json)
        .bind(policy.priority)
        .bind(policy.is_active)
        .bind(policy.audit_info.created_at)
        .bind(policy.audit_info.created_by.as_ref().map(|u| u.0))
        .bind(policy.audit_info.updated_at)
        .bind(policy.audit_info.updated_by.as_ref().map(|u| u.0))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn update(&self, policy: &Policy) -> AppResult<()> {
        let subjects_json = serde_json::to_value(&policy.subjects).map_err(map_json_error)?;
        let resources_json = serde_json::to_value(&policy.resources).map_err(map_json_error)?;
        let actions_json = serde_json::to_value(&policy.actions).map_err(map_json_error)?;
        let conditions_json: Option<serde_json::Value> = policy
            .conditions
            .as_ref()
            .map(|c| serde_json::from_str(c))
            .transpose()
            .map_err(map_json_error)?;

        sqlx::query(
            r#"
            UPDATE policies
            SET name = $2, description = $3, effect = $4, subjects = $5, resources = $6, 
                actions = $7, conditions = $8, priority = $9, is_active = $10, 
                updated_at = $11, updated_by = $12
            WHERE id = $1
            "#,
        )
        .bind(policy.id.0)
        .bind(&policy.name)
        .bind(&policy.description)
        .bind(policy.effect.to_string())
        .bind(&subjects_json)
        .bind(&resources_json)
        .bind(&actions_json)
        .bind(&conditions_json)
        .bind(policy.priority)
        .bind(policy.is_active)
        .bind(policy.audit_info.updated_at)
        .bind(policy.audit_info.updated_by.as_ref().map(|u| u.0))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(&self, id: &PolicyId) -> AppResult<()> {
        sqlx::query("DELETE FROM policies WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id(&self, id: &PolicyId) -> AppResult<Option<Policy>> {
        let row = sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT id, tenant_id, name, description, effect, subjects, resources, actions, 
                   conditions, priority, is_active, created_at, created_by, updated_at, updated_by
            FROM policies WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|r| r.into()))
    }

    async fn list_by_tenant(&self, tenant_id: &TenantId, page: u32, page_size: u32) -> AppResult<(Vec<Policy>, i64)> {
        let offset = (page.saturating_sub(1)) * page_size;

        let rows = sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT id, tenant_id, name, description, effect, subjects, resources, actions, 
                   conditions, priority, is_active, created_at, created_by, updated_at, updated_by
            FROM policies 
            WHERE tenant_id = $1 
            ORDER BY priority DESC, created_at DESC 
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id.0)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM policies WHERE tenant_id = $1")
            .bind(tenant_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(|r| r.into()).collect(), total.0))
    }

    async fn list_active_by_tenant(&self, tenant_id: &TenantId) -> AppResult<Vec<Policy>> {
        let rows = sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT id, tenant_id, name, description, effect, subjects, resources, actions, 
                   conditions, priority, is_active, created_at, created_by, updated_at, updated_by
            FROM policies 
            WHERE tenant_id = $1 AND is_active = TRUE
            ORDER BY priority DESC
            "#,
        )
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_subject(&self, tenant_id: &TenantId, subject: &str) -> AppResult<Vec<Policy>> {
        let rows = sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT id, tenant_id, name, description, effect, subjects, resources, actions, 
                   conditions, priority, is_active, created_at, created_by, updated_at, updated_by
            FROM policies 
            WHERE tenant_id = $1 AND is_active = TRUE AND subjects @> $2::jsonb
            ORDER BY priority DESC
            "#,
        )
        .bind(tenant_id.0)
        .bind(serde_json::json!([subject]))
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_resource(&self, tenant_id: &TenantId, resource: &str) -> AppResult<Vec<Policy>> {
        let rows = sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT id, tenant_id, name, description, effect, subjects, resources, actions, 
                   conditions, priority, is_active, created_at, created_by, updated_at, updated_by
            FROM policies 
            WHERE tenant_id = $1 AND is_active = TRUE AND resources @> $2::jsonb
            ORDER BY priority DESC
            "#,
        )
        .bind(tenant_id.0)
        .bind(serde_json::json!([resource]))
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn exists_by_name(&self, tenant_id: &TenantId, name: &str) -> AppResult<bool> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM policies WHERE tenant_id = $1 AND name = $2)",
        )
        .bind(tenant_id.0)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.0)
    }
}

// ============ 数据行映射 ============

#[derive(sqlx::FromRow)]
struct PolicyRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: Option<String>,
    effect: String,
    subjects: serde_json::Value,
    resources: serde_json::Value,
    actions: serde_json::Value,
    conditions: Option<serde_json::Value>,
    priority: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
    created_by: Option<Uuid>,
    updated_at: DateTime<Utc>,
    updated_by: Option<Uuid>,
}

impl From<PolicyRow> for Policy {
    fn from(row: PolicyRow) -> Self {
        let effect = row.effect.parse().unwrap_or(Effect::Allow);
        let subjects: Vec<String> = serde_json::from_value(row.subjects).unwrap_or_default();
        let resources: Vec<String> = serde_json::from_value(row.resources).unwrap_or_default();
        let actions: Vec<String> = serde_json::from_value(row.actions).unwrap_or_default();
        let conditions = row.conditions.map(|c| c.to_string());

        Policy {
            id: PolicyId::from_uuid(row.id),
            tenant_id: TenantId::from_uuid(row.tenant_id),
            name: row.name,
            description: row.description,
            effect,
            subjects,
            resources,
            actions,
            conditions,
            priority: row.priority,
            is_active: row.is_active,
            audit_info: AuditInfo {
                created_at: row.created_at,
                created_by: row.created_by.map(cuba_common::UserId::from_uuid),
                updated_at: row.updated_at,
                updated_by: row.updated_by.map(cuba_common::UserId::from_uuid),
            },
        }
    }
}
