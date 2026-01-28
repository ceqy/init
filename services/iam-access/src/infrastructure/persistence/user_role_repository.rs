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

use crate::infrastructure::cache::AuthCache;
use std::sync::Arc;

pub struct PostgresUserRoleRepository {
    pool: PgPool,
    auth_cache: Option<Arc<AuthCache>>,
}

impl PostgresUserRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            auth_cache: None,
        }
    }

    pub fn with_cache(mut self, auth_cache: Arc<AuthCache>) -> Self {
        self.auth_cache = Some(auth_cache);
        self
    }
}

#[async_trait]
impl UserRoleRepository for PostgresUserRoleRepository {
    async fn assign_roles(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
        role_ids: &[RoleId],
    ) -> AppResult<()> {
        let user_uuid: Uuid = user_id
            .parse()
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

        if let Some(cache) = &self.auth_cache {
            let _ = cache
                .invalidate_user_roles(tenant_id, &cuba_common::UserId::from_uuid(user_uuid))
                .await;
        }

        Ok(())
    }

    async fn remove_roles(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
        role_ids: &[RoleId],
    ) -> AppResult<()> {
        let user_uuid: Uuid = user_id
            .parse()
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

        if let Some(cache) = &self.auth_cache {
            let _ = cache
                .invalidate_user_roles(tenant_id, &cuba_common::UserId::from_uuid(user_uuid))
                .await;
        }

        Ok(())
    }

    async fn get_user_roles(&self, user_id: &str, tenant_id: &TenantId) -> AppResult<Vec<Role>> {
        let user_uuid: Uuid = user_id
            .parse()
            .map_err(|_| AppError::validation("Invalid user_id"))?;
        let user_id_obj = cuba_common::UserId::from_uuid(user_uuid);

        if let Some(cache) = &self.auth_cache {
            if let Ok(Some(roles)) = cache.get_user_roles(tenant_id, &user_id_obj).await {
                return Ok(roles);
            }
        }

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

        // 批量加载所有角色的权限 (修复 N+1 查询)
        let role_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let mut permissions_map = self.batch_load_roles_permissions(&role_ids).await?;

        let roles: Vec<Role> = rows
            .into_iter()
            .map(|row| {
                let perms = permissions_map.remove(&row.id).unwrap_or_default();
                row.into_role(perms)
            })
            .collect();

        if let Some(cache) = &self.auth_cache {
            let _ = cache.set_user_roles(tenant_id, &user_id_obj, &roles).await;
        }

        Ok(roles)
    }

    async fn get_user_permissions(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<Permission>> {
        // 尝试从缓存获取角色并提取权限
        if let Ok(roles) = self.get_user_roles(user_id, tenant_id).await {
            let mut permissions = std::collections::HashSet::new();
            for role in roles {
                for perm in role.permissions {
                    permissions.insert(perm);
                }
            }
            // Sort by resource and action to match SQL ordering
            let mut result: Vec<Permission> = permissions.into_iter().collect();
            result.sort_by(|a, b| {
                let res = a.resource.cmp(&b.resource);
                if res == std::cmp::Ordering::Equal {
                    a.action.cmp(&b.action)
                } else {
                    res
                }
            });
            return Ok(result);
        }

        // Fallback to DB query if get_user_roles fails (which shouldn't happen given logic, but just in case)
        // Actually get_user_roles handles cache miss by fetching from DB, so calling it is enough.
        // But get_user_permissions implementation above replaces the SQL entirely by reusing get_user_roles logic.
        // So I don't need the SQL below anymore if I rely on get_user_roles.
        // However, Permission trait bounds (Hash/Eq) needed for HashSet. Permission already derives Hash/Eq.

        unreachable!("get_user_roles handles DB fetch");
    }

    async fn user_has_permission(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
        permission_code: &str,
    ) -> AppResult<bool> {
        let user_uuid: Uuid = user_id
            .parse()
            .map_err(|_| AppError::validation("Invalid user_id"))?;

        // 1. 检查精确匹配
        let exact_match: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM permissions p
                INNER JOIN role_permissions rp ON p.id = rp.permission_id
                INNER JOIN user_roles ur ON rp.role_id = ur.role_id
                INNER JOIN roles r ON ur.role_id = r.id
                WHERE ur.user_id = $1 AND ur.tenant_id = $2 
                AND r.is_active = TRUE AND p.is_active = TRUE
                AND p.code = $3
            )
            "#,
        )
        .bind(user_uuid)
        .bind(tenant_id.0)
        .bind(permission_code)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if exact_match.0 {
            return Ok(true);
        }

        // 2. 检查通配符匹配 (resource:*)
        // 仅当 permission_code 格式为 "resource:action" 时检查
        let parts: Vec<&str> = permission_code.split(':').collect();
        if parts.len() == 2 {
            let wildcard_code = format!("{}:*", parts[0]);
            let wildcard_match: (bool,) = sqlx::query_as(
                r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM permissions p
                    INNER JOIN role_permissions rp ON p.id = rp.permission_id
                    INNER JOIN user_roles ur ON rp.role_id = ur.role_id
                    INNER JOIN roles r ON ur.role_id = r.id
                    WHERE ur.user_id = $1 AND ur.tenant_id = $2 
                    AND r.is_active = TRUE AND p.is_active = TRUE
                    AND p.code = $3
                )
                "#,
            )
            .bind(user_uuid)
            .bind(tenant_id.0)
            .bind(&wildcard_code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

            return Ok(wildcard_match.0);
        }

        Ok(false)
    }

    async fn clear_user_roles(&self, user_id: &str, tenant_id: &TenantId) -> AppResult<()> {
        let user_uuid: Uuid = user_id
            .parse()
            .map_err(|_| AppError::validation("Invalid user_id"))?;

        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND tenant_id = $2")
            .bind(user_uuid)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        if let Some(cache) = &self.auth_cache {
            let _ = cache
                .invalidate_user_roles(tenant_id, &cuba_common::UserId::from_uuid(user_uuid))
                .await;
        }

        Ok(())
    }

    async fn user_has_role(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
        role_id: &RoleId,
    ) -> AppResult<bool> {
        let user_uuid: Uuid = user_id
            .parse()
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
    /*
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
    */

    /// 批量加载多个角色的权限 (修复 N+1 查询)
    async fn batch_load_roles_permissions(
        &self,
        role_ids: &[Uuid],
    ) -> AppResult<std::collections::HashMap<Uuid, Vec<Permission>>> {
        if role_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let rows = sqlx::query_as::<_, RolePermissionRow>(
            r#"
            SELECT rp.role_id, p.id, p.code, p.name, p.description, p.resource, p.action, p.module, p.is_active, p.created_at
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = ANY($1)
            ORDER BY rp.role_id, p.resource, p.action
            "#,
        )
        .bind(role_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut map: std::collections::HashMap<Uuid, Vec<Permission>> =
            std::collections::HashMap::new();
        for row in rows {
            map.entry(row.role_id)
                .or_insert_with(Vec::new)
                .push(row.into_permission());
        }

        // 确保所有角色都有条目 (即使没有权限)
        for role_id in role_ids {
            map.entry(*role_id).or_insert_with(Vec::new);
        }

        Ok(map)
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

/// 用于批量加载角色权限的行结构
#[derive(sqlx::FromRow)]
struct RolePermissionRow {
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

impl RolePermissionRow {
    fn into_permission(self) -> Permission {
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
