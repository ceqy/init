//! PostgreSQL 角色权限关联仓储实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::role::{Permission, PermissionId, RoleId, RolePermissionRepository};

/// 将 sqlx 错误转换为 AppError
fn map_sqlx_error(e: sqlx::Error) -> AppError {
    AppError::database(e.to_string())
}

pub struct PostgresRolePermissionRepository {
    pool: PgPool,
}

impl PostgresRolePermissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RolePermissionRepository for PostgresRolePermissionRepository {
    async fn assign_permissions(&self, role_id: &RoleId, permission_ids: &[PermissionId]) -> AppResult<()> {
        // 使用事务保证原子性
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        
        for perm_id in permission_ids {
            sqlx::query(
                r#"
                INSERT INTO role_permissions (role_id, permission_id, assigned_at)
                VALUES ($1, $2, $3)
                ON CONFLICT (role_id, permission_id) DO NOTHING
                "#,
            )
            .bind(role_id.0)
            .bind(perm_id.0)
            .bind(Utc::now())
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn remove_permissions(&self, role_id: &RoleId, permission_ids: &[PermissionId]) -> AppResult<()> {
        let perm_uuids: Vec<Uuid> = permission_ids.iter().map(|p| p.0).collect();

        sqlx::query(
            "DELETE FROM role_permissions WHERE role_id = $1 AND permission_id = ANY($2)",
        )
        .bind(role_id.0)
        .bind(&perm_uuids)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn get_role_permissions(&self, role_id: &RoleId) -> AppResult<Vec<Permission>> {
        let rows = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT p.id, p.code, p.name, p.description, p.resource, p.action, p.module, p.is_active, p.created_at
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            ORDER BY p.resource, p.action
            "#,
        )
        .bind(role_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn clear_role_permissions(&self, role_id: &RoleId) -> AppResult<()> {
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role_id.0)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
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
    created_at: DateTime<Utc>,
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
