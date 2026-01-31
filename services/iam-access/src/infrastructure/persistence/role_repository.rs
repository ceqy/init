//! PostgreSQL 角色仓储实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{AuditInfo, TenantId};
use errors::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::role::{Permission, PermissionId, Role, RoleId, RoleRepository};

/// 将 sqlx 错误转换为 AppError
fn map_sqlx_error(e: sqlx::Error) -> AppError {
    AppError::database(e.to_string())
}

pub struct PostgresRoleRepository {
    pool: PgPool,
}

impl PostgresRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepository for PostgresRoleRepository {
    async fn create(&self, role: &Role) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO roles (id, tenant_id, code, name, description, is_system, is_active, created_at, created_by, updated_at, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(role.id.0)
        .bind(role.tenant_id.0)
        .bind(&role.code)
        .bind(&role.name)
        .bind(&role.description)
        .bind(role.is_system)
        .bind(role.is_active)
        .bind(role.audit_info.created_at)
        .bind(role.audit_info.created_by.as_ref().map(|u| u.0))
        .bind(role.audit_info.updated_at)
        .bind(role.audit_info.updated_by.as_ref().map(|u| u.0))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn update(&self, role: &Role) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE roles
            SET name = $2, description = $3, is_active = $4, updated_at = $5, updated_by = $6
            WHERE id = $1
            "#,
        )
        .bind(role.id.0)
        .bind(&role.name)
        .bind(&role.description)
        .bind(role.is_active)
        .bind(role.audit_info.updated_at)
        .bind(role.audit_info.updated_by.as_ref().map(|u| u.0))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(&self, id: &RoleId) -> AppResult<()> {
        sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id(&self, id: &RoleId) -> AppResult<Option<Role>> {
        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, tenant_id, code, name, description, is_system, is_active, 
                   created_at, created_by, updated_at, updated_by
            FROM roles WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(r) => {
                let permissions = self.load_role_permissions(&r.id).await?;
                Ok(Some(r.into_role(permissions)))
            }
            None => Ok(None),
        }
    }

    async fn find_by_code(&self, tenant_id: &TenantId, code: &str) -> AppResult<Option<Role>> {
        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, tenant_id, code, name, description, is_system, is_active, 
                   created_at, created_by, updated_at, updated_by
            FROM roles WHERE tenant_id = $1 AND code = $2
            "#,
        )
        .bind(tenant_id.0)
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(r) => {
                let permissions = self.load_role_permissions(&r.id).await?;
                Ok(Some(r.into_role(permissions)))
            }
            None => Ok(None),
        }
    }

    async fn list_by_tenant(
        &self,
        tenant_id: &TenantId,
        page: u32,
        page_size: u32,
    ) -> AppResult<(Vec<Role>, i64)> {
        let offset = (page.saturating_sub(1)) * page_size;

        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, tenant_id, code, name, description, is_system, is_active, 
                   created_at, created_by, updated_at, updated_by
            FROM roles 
            WHERE tenant_id = $1 
            ORDER BY is_system DESC, created_at DESC 
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id.0)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM roles WHERE tenant_id = $1")
            .bind(tenant_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        if rows.is_empty() {
            return Ok((Vec::new(), total.0));
        }

        let role_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let permissions_map = self.load_roles_permissions(&role_ids).await?;

        let roles = rows
            .into_iter()
            .map(|r| {
                let permissions = permissions_map.get(&r.id).cloned().unwrap_or_default();
                r.into_role(permissions)
            })
            .collect();

        Ok((roles, total.0))
    }

    async fn search(
        &self,
        tenant_id: &TenantId,
        query: &str,
        page: u32,
        page_size: u32,
    ) -> AppResult<(Vec<Role>, i64)> {
        let offset = (page.saturating_sub(1)) * page_size;
        let search_pattern = format!("%{}%", query);

        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, tenant_id, code, name, description, is_system, is_active, 
                   created_at, created_by, updated_at, updated_by
            FROM roles 
            WHERE tenant_id = $1 AND (code ILIKE $2 OR name ILIKE $2)
            ORDER BY is_system DESC, created_at DESC 
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id.0)
        .bind(&search_pattern)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM roles WHERE tenant_id = $1 AND (code ILIKE $2 OR name ILIKE $2)",
        )
        .bind(tenant_id.0)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if rows.is_empty() {
            return Ok((Vec::new(), total.0));
        }

        let role_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let permissions_map = self.load_roles_permissions(&role_ids).await?;

        let roles = rows
            .into_iter()
            .map(|r| {
                let permissions = permissions_map.get(&r.id).cloned().unwrap_or_default();
                r.into_role(permissions)
            })
            .collect();

        Ok((roles, total.0))
    }

    async fn exists_by_code(&self, tenant_id: &TenantId, code: &str) -> AppResult<bool> {
        let result: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM roles WHERE tenant_id = $1 AND code = $2)")
                .bind(tenant_id.0)
                .bind(code)
                .fetch_one(&self.pool)
                .await
                .map_err(map_sqlx_error)?;

        Ok(result.0)
    }

    async fn list_active_tenants(&self) -> AppResult<Vec<TenantId>> {
        let rows = sqlx::query_as::<_, TenantIdRow>("SELECT DISTINCT tenant_id FROM roles")
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|r| TenantId::from_uuid(r.tenant_id))
            .collect())
    }
}

#[derive(sqlx::FromRow)]
struct TenantIdRow {
    tenant_id: Uuid,
}

impl PostgresRoleRepository {
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
    async fn load_roles_permissions(
        &self,
        role_ids: &[Uuid],
    ) -> AppResult<std::collections::HashMap<Uuid, Vec<Permission>>> {
        let rows = sqlx::query_as::<_, RolePermissionJoinRow>(
            r#"
            SELECT rp.role_id, p.id, p.code, p.name, p.description, p.resource, p.action, p.module, p.is_active, p.created_at
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = ANY($1)
            "#,
        )
        .bind(role_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut map: std::collections::HashMap<Uuid, Vec<Permission>> =
            std::collections::HashMap::new();
        for row in rows {
            map.entry(row.role_id).or_default().push(row.permission());
        }

        Ok(map)
    }
}

// ============ 数据行映射 ============

#[derive(sqlx::FromRow)]
struct RolePermissionJoinRow {
    role_id: Uuid,
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

impl RolePermissionJoinRow {
    fn permission(self) -> Permission {
        Permission {
            id: PermissionId::from_uuid(self.id),
            code: self.code,
            name: self.name,
            description: self.description,
            resource: self.resource,
            action: self.action,
            module: self.module,
            is_active: self.is_active,
            created_at: self.created_at,
        }
    }
}

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
                created_by: self.created_by.map(common::UserId::from_uuid),
                updated_at: self.updated_at,
                updated_by: self.updated_by.map(common::UserId::from_uuid),
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
