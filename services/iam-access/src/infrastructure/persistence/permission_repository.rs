//! PostgreSQL 权限仓储实现

use async_trait::async_trait;
use chrono::Utc;
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::role::{Permission, PermissionId, PermissionRepository};

/// 将 sqlx 错误转换为 AppError
fn map_sqlx_error(e: sqlx::Error) -> AppError {
    AppError::database(e.to_string())
}

pub struct PostgresPermissionRepository {
    pool: PgPool,
}

impl PostgresPermissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PermissionRepository for PostgresPermissionRepository {
    async fn create(&self, permission: &Permission) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO permissions (id, code, name, description, resource, action, module, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(permission.id.0)
        .bind(&permission.code)
        .bind(&permission.name)
        .bind(&permission.description)
        .bind(&permission.resource)
        .bind(&permission.action)
        .bind(&permission.module)
        .bind(permission.is_active)
        .bind(permission.created_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn update(&self, permission: &Permission) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE permissions
            SET name = $2, description = $3, is_active = $4
            WHERE id = $1
            "#,
        )
        .bind(permission.id.0)
        .bind(&permission.name)
        .bind(&permission.description)
        .bind(permission.is_active)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(&self, id: &PermissionId) -> AppResult<()> {
        sqlx::query("DELETE FROM permissions WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id(&self, id: &PermissionId) -> AppResult<Option<Permission>> {
        let row = sqlx::query_as::<_, PermissionRow>(
            "SELECT id, code, name, description, resource, action, module, is_active, created_at FROM permissions WHERE id = $1",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_code(&self, code: &str) -> AppResult<Option<Permission>> {
        let row = sqlx::query_as::<_, PermissionRow>(
            "SELECT id, code, name, description, resource, action, module, is_active, created_at FROM permissions WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_ids(&self, ids: &[PermissionId]) -> AppResult<Vec<Permission>> {
        let uuids: Vec<Uuid> = ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query_as::<_, PermissionRow>(
            "SELECT id, code, name, description, resource, action, module, is_active, created_at FROM permissions WHERE id = ANY($1)",
        )
        .bind(&uuids)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn list_all(&self, page: u32, page_size: u32) -> AppResult<(Vec<Permission>, i64)> {
        let offset = (page.saturating_sub(1)) * page_size;

        let rows = sqlx::query_as::<_, PermissionRow>(
            "SELECT id, code, name, description, resource, action, module, is_active, created_at FROM permissions ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM permissions")
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(|r| r.into()).collect(), total.0))
    }

    async fn list_by_module(&self, module: &str) -> AppResult<Vec<Permission>> {
        let rows = sqlx::query_as::<_, PermissionRow>(
            "SELECT id, code, name, description, resource, action, module, is_active, created_at FROM permissions WHERE module = $1 ORDER BY resource, action",
        )
        .bind(module)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn list_by_resource(&self, resource: &str) -> AppResult<Vec<Permission>> {
        let rows = sqlx::query_as::<_, PermissionRow>(
            "SELECT id, code, name, description, resource, action, module, is_active, created_at FROM permissions WHERE resource = $1 ORDER BY action",
        )
        .bind(resource)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn exists_by_code(&self, code: &str) -> AppResult<bool> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM permissions WHERE code = $1)",
        )
        .bind(code)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.0)
    }
}

// ============ 数据行映射 ============

#[derive(sqlx::FromRow)]
struct PermissionRow {
    id: Uuid,
    code: String,
    name: String,
    description: Option<String>,
    resource: String,
    action: String,
    module: String,
    is_active: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<PermissionRow> for Permission {
    fn from(row: PermissionRow) -> Self {
        Permission {
            id: PermissionId::from_uuid(row.id),
            code: row.code,
            name: row.name,
            description: row.description,
            resource: row.resource,
            action: row.action,
            module: row.module,
            is_active: row.is_active,
            created_at: row.created_at,
        }
    }
}
