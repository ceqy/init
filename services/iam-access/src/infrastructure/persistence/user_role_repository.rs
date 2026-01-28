//! PostgreSQL 用户角色仓储实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cuba_common::{AuditInfo, TenantId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::role::{Permission, PermissionId, Role, RoleId, UserRoleRepository};

/// 将 sqlx 错误转换为 AppError
fn map_sqlx_error(e: sqlx::Error) -> AppError {
    AppError::database(e.to_string())
}

pub struct PostgresUserRoleRepository {
    pool: PgPool,
}

impl PostgresUserRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRoleRepository for PostgresUserRoleRepository {
    async fn assign_roles(&self, user_id: &str, tenant_id: &TenantId, role_ids: &[RoleId]) -> AppResult<()> {
        let user_uuid: Uuid = user_id.parse()
            .map_err(|_| AppError::validation("Invalid user_id"))?;

        for role_id in role_ids {
            sqlx::query(
                r#"
                INSERT INTO user_roles (user_id, tenant_id, role_id, assigned_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (user_id, tenant_id, role_id) DO NOTHING
                "#,
            )
            .bind(user_uuid)
            .bind(tenant_id.0)
            .bind(role_id.0)
            .bind(Utc::now())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        }

        Ok(())
    }

    async fn remove_roles(&self, user_id: &str, tenant_id: &TenantId, role_ids: &[RoleId]) -> AppResult<()> {
        let user_uuid: Uuid = user_id.parse()
            .map_err(|_| AppError::validation("Invalid user_id"))?;

        let role_uuids: Vec<Uuid> = role_ids.iter().map(|r| r.0).collect();

        sqlx::query(
            "DELETE FROM user_roles WHERE user_id = $1 AND tenant_id = $2 AND role_id = ANY($3)",
        )
        .bind(user_uuid)
        .bind(tenant_id.0)
        .bind(&role_uuids)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn get_user_roles(&self, user_id: &str, tenant_id: &TenantId) -> AppResult<Vec<Role>> {
        let user_uuid: Uuid = user_id.parse()
            .map_err(|_| AppError::validation("Invalid user_id"))?;

        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT r.id, r.tenant_id, r.code, r.name, r.description, r.is_system, r.is_active,
                   r.created_at, r.created_by, r.updated_at, r.updated_by
            FROM roles r
            INNER JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1 AND ur.tenant_id = $2 AND r.is_active = TRUE
            ORDER BY r.is_system DESC, r.name
            "#,
        )
        .bind(user_uuid)
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        // 加载每个角色的权限
        let mut roles = Vec::new();
        for row in rows {
            let permissions = self.load_role_permissions(&row.id).await?;
            roles.push(row.into_role(permissions));
        }

        Ok(roles)
    }

    async fn get_user_permissions(&self, user_id: &str, tenant_id: &TenantId) -> AppResult<Vec<Permission>> {
        let user_uuid: Uuid = user_id.parse()
            .map_err(|_| AppError::validation("Invalid user_id"))?;

        let rows = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT DISTINCT p.id, p.code, p.name, p.description, p.resource, p.action, p.module, p.is_active, p.created_at
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            INNER JOIN user_roles ur ON rp.role_id = ur.role_id
            INNER JOIN roles r ON ur.role_id = r.id
            WHERE ur.user_id = $1 AND ur.tenant_id = $2 AND r.is_active = TRUE AND p.is_active = TRUE
            ORDER BY p.resource, p.action
            "#,
        )
        .bind(user_uuid)
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn user_has_permission(&self, user_id: &str, tenant_id: &TenantId, permission_code: &str) -> AppResult<bool> {
        let user_uuid: Uuid = user_id.parse()
            .map_err(|_| AppError::validation("Invalid user_id"))?;

        let result: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM permissions p
                INNER JOIN role_permissions rp ON p.id = rp.permission_id
                INNER JOIN user_roles ur ON rp.role_id = ur.role_id
                INNER JOIN roles r ON ur.role_id = r.id
                WHERE ur.user_id = $1 AND ur.tenant_id = $2 
                AND r.is_active = TRUE AND p.is_active = TRUE
                AND (p.code = $3 OR p.code LIKE REPLACE($3, ':', ':%'))
            )
            "#,
        )
        .bind(user_uuid)
        .bind(tenant_id.0)
        .bind(permission_code)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.0)
    }

    async fn clear_user_roles(&self, user_id: &str, tenant_id: &TenantId) -> AppResult<()> {
        let user_uuid: Uuid = user_id.parse()
            .map_err(|_| AppError::validation("Invalid user_id"))?;

        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND tenant_id = $2")
            .bind(user_uuid)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn user_has_role(&self, user_id: &str, tenant_id: &TenantId, role_id: &RoleId) -> AppResult<bool> {
        let user_uuid: Uuid = user_id.parse()
            .map_err(|_| AppError::validation("Invalid user_id"))?;

        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM user_roles WHERE user_id = $1 AND tenant_id = $2 AND role_id = $3)",
        )
        .bind(user_uuid)
        .bind(tenant_id.0)
        .bind(role_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.0)
    }
}

impl PostgresUserRoleRepository {
    async fn load_role_permissions(&self, role_id: &Uuid) -> AppResult<Vec<Permission>> {
        let rows = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT p.id, p.code, p.name, p.description, p.resource, p.action, p.module, p.is_active, p.created_at
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            ORDER BY p.resource, p.action
            "#,
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

// ============ 数据行映射 ============

#[derive(sqlx::FromRow)]
struct RoleRow {
    id: Uuid,
    tenant_id: Uuid,
    code: String,
    name: String,
    description: Option<String>,
    is_system: bool,
    is_active: bool,
    created_at: DateTime<Utc>,
    created_by: Option<Uuid>,
    updated_at: DateTime<Utc>,
    updated_by: Option<Uuid>,
}

impl RoleRow {
    fn into_role(self, permissions: Vec<Permission>) -> Role {
        Role {
            id: RoleId::from_uuid(self.id),
            tenant_id: TenantId::from_uuid(self.tenant_id),
            code: self.code,
            name: self.name,
            description: self.description,
            is_system: self.is_system,
            is_active: self.is_active,
            permissions,
            audit_info: AuditInfo {
                created_at: self.created_at,
                created_by: self.created_by.map(cuba_common::UserId::from_uuid),
                updated_at: self.updated_at,
                updated_by: self.updated_by.map(cuba_common::UserId::from_uuid),
            },
        }
    }
}

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
