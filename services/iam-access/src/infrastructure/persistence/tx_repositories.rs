//! Transactional repositories for iam-access
//!
//! These repositories use a shared transaction instead of a connection pool.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cuba_common::{AuditInfo, TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::{Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::error_mapper::map_sqlx_error;
use crate::domain::policy::{Effect, Policy, PolicyId, PolicyRepository};
/// 将 serde_json 错误转换为 AppError
fn map_json_error(e: serde_json::Error) -> AppError {
    AppError::internal(format!("JSON serialization error: {}", e))
}

use crate::domain::role::{
    Permission, PermissionId, PermissionRepository, Role, RoleId, RolePermissionRepository,
    RoleRepository, UserRoleRepository,
};

/// Shared transaction type
pub type SharedTx = Arc<Mutex<Option<Transaction<'static, Postgres>>>>;

/// Macro to define a TxRepository structure
macro_rules! define_tx_repo {
    ($name:ident) => {
        pub struct $name {
            tx: SharedTx,
        }

        impl $name {
            pub fn new(tx: SharedTx) -> Self {
                Self { tx }
            }
        }
    };
}

define_tx_repo!(TxRoleRepository);
define_tx_repo!(TxPermissionRepository);
define_tx_repo!(TxRolePermissionRepository);
define_tx_repo!(TxUserRoleRepository);
define_tx_repo!(TxPolicyRepository);

#[async_trait]
impl RoleRepository for TxRoleRepository {
    async fn create(&self, role: &Role) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

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
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn update(&self, role: &Role) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

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
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(&self, id: &RoleId) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id.0)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id(&self, id: &RoleId) -> AppResult<Option<Role>> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, tenant_id, code, name, description, is_system, is_active, 
                   created_at, created_by, updated_at, updated_by
            FROM roles WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(r) => {
                let permissions = self.load_role_permissions_in_tx(&r.id, tx).await?;
                Ok(Some(r.into_role(permissions)))
            }
            None => Ok(None),
        }
    }

    async fn find_by_code(&self, tenant_id: &TenantId, code: &str) -> AppResult<Option<Role>> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, tenant_id, code, name, description, is_system, is_active, 
                   created_at, created_by, updated_at, updated_by
            FROM roles WHERE tenant_id = $1 AND code = $2
            "#,
        )
        .bind(tenant_id.0)
        .bind(code)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(r) => {
                let permissions = self.load_role_permissions_in_tx(&r.id, tx).await?;
                Ok(Some(r.into_role(permissions)))
            }
            None => Ok(None),
        }
    }

    async fn list_by_tenant(
        &self,
        _tenant_id: &TenantId,
        _page: u32,
        _page_size: u32,
    ) -> AppResult<(Vec<Role>, i64)> {
        Err(AppError::internal(
            "list_by_tenant not implemented for TxRoleRepository yet",
        ))
    }

    async fn search(
        &self,
        _tenant_id: &TenantId,
        _query: &str,
        _page: u32,
        _page_size: u32,
    ) -> AppResult<(Vec<Role>, i64)> {
        Err(AppError::internal(
            "search not implemented for TxRoleRepository yet",
        ))
    }

    async fn exists_by_code(&self, tenant_id: &TenantId, code: &str) -> AppResult<bool> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        let row: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM roles WHERE tenant_id = $1 AND code = $2)")
                .bind(tenant_id.0)
                .bind(code)
                .fetch_one(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;

        Ok(row.0)
    }

    async fn list_active_tenants(&self) -> AppResult<Vec<TenantId>> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        let rows = sqlx::query_as::<_, TenantIdRow>("SELECT DISTINCT tenant_id FROM roles")
            .fetch_all(&mut **tx)
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

impl TxRoleRepository {
    async fn load_role_permissions_in_tx<'a>(
        &self,
        role_id: &Uuid,
        tx: &mut Transaction<'a, Postgres>,
    ) -> AppResult<Vec<Permission>> {
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
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[async_trait]
impl PermissionRepository for TxPermissionRepository {
    async fn create(&self, permission: &Permission) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        sqlx::query(
            r#"
            INSERT INTO permissions (id, code, name, description, module, resource, action, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(permission.id.0)
        .bind(&permission.code)
        .bind(&permission.name)
        .bind(&permission.description)
        .bind(&permission.module)
        .bind(&permission.resource)
        .bind(&permission.action)
        .bind(permission.is_active)
        .bind(permission.created_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn update(&self, permission: &Permission) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        sqlx::query(
            r#"
            UPDATE permissions
            SET name = $2, description = $3, module = $4, resource = $5, action = $6, is_active = $7
            WHERE id = $1
            "#,
        )
        .bind(permission.id.0)
        .bind(&permission.name)
        .bind(&permission.description)
        .bind(&permission.module)
        .bind(&permission.resource)
        .bind(&permission.action)
        .bind(permission.is_active)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(&self, id: &PermissionId) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        sqlx::query("DELETE FROM permissions WHERE id = $1")
            .bind(id.0)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id(&self, _id: &PermissionId) -> AppResult<Option<Permission>> {
        Err(AppError::internal("not implemented"))
    }

    async fn find_by_code(&self, _code: &str) -> AppResult<Option<Permission>> {
        Err(AppError::internal("not implemented"))
    }

    async fn find_by_ids(&self, _ids: &[PermissionId]) -> AppResult<Vec<Permission>> {
        Err(AppError::internal("not implemented"))
    }

    async fn list_all(&self, _page: u32, _page_size: u32) -> AppResult<(Vec<Permission>, i64)> {
        Err(AppError::internal("not implemented"))
    }

    async fn list_by_module(&self, _module: &str) -> AppResult<Vec<Permission>> {
        Err(AppError::internal("not implemented"))
    }

    async fn list_by_resource(&self, _resource: &str) -> AppResult<Vec<Permission>> {
        Err(AppError::internal("not implemented"))
    }

    async fn exists_by_code(&self, code: &str) -> AppResult<bool> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        let row: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM permissions WHERE code = $1)")
                .bind(code)
                .fetch_one(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;

        Ok(row.0)
    }
}

// Implement RolePermissionRepository and UserRoleRepository placeholders
#[async_trait]
impl RolePermissionRepository for TxRolePermissionRepository {
    async fn assign_permissions(
        &self,
        role_id: &RoleId,
        permission_ids: &[PermissionId],
    ) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        for perm_id in permission_ids {
            sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)")
                .bind(role_id.0)
                .bind(perm_id.0)
                .execute(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn remove_permissions(
        &self,
        role_id: &RoleId,
        permission_ids: &[PermissionId],
    ) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        for perm_id in permission_ids {
            sqlx::query("DELETE FROM role_permissions WHERE role_id = $1 AND permission_id = $2")
                .bind(role_id.0)
                .bind(perm_id.0)
                .execute(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn get_role_permissions(&self, _role_id: &RoleId) -> AppResult<Vec<Permission>> {
        Err(AppError::internal("not implemented"))
    }

    async fn clear_role_permissions(&self, role_id: &RoleId) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role_id.0)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }
}

#[async_trait]
impl UserRoleRepository for TxUserRoleRepository {
    async fn assign_roles(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
        role_ids: &[RoleId],
    ) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        for role_id in role_ids {
            sqlx::query("INSERT INTO user_roles (user_id, tenant_id, role_id) VALUES ($1, $2, $3)")
                .bind(user_id)
                .bind(tenant_id.0)
                .bind(role_id.0)
                .execute(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn remove_roles(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
        role_ids: &[RoleId],
    ) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        for role_id in role_ids {
            sqlx::query(
                "DELETE FROM user_roles WHERE user_id = $1 AND tenant_id = $2 AND role_id = $3",
            )
            .bind(user_id)
            .bind(tenant_id.0)
            .bind(role_id.0)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn get_user_roles(&self, _user_id: &str, _tenant_id: &TenantId) -> AppResult<Vec<Role>> {
        Err(AppError::internal("not implemented"))
    }

    async fn get_user_permissions(
        &self,
        _user_id: &str,
        _tenant_id: &TenantId,
    ) -> AppResult<Vec<Permission>> {
        Err(AppError::internal("not implemented"))
    }

    async fn clear_user_roles(&self, user_id: &str, tenant_id: &TenantId) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND tenant_id = $2")
            .bind(user_id)
            .bind(tenant_id.0)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn user_has_role(
        &self,
        _user_id: &str,
        _tenant_id: &TenantId,
        _role_id: &RoleId,
    ) -> AppResult<bool> {
        Err(AppError::internal("not implemented"))
    }

    async fn user_has_permission(
        &self,
        _user_id: &str,
        _tenant_id: &TenantId,
        _permission_code: &str,
    ) -> AppResult<bool> {
        Err(AppError::internal("not implemented"))
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
                created_by: self.created_by.map(UserId::from_uuid),
                updated_at: self.updated_at,
                updated_by: self.updated_by.map(UserId::from_uuid),
            },
        }
    }
}

#[async_trait]
impl PolicyRepository for TxPolicyRepository {
    async fn create(&self, policy: &Policy) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

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
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn update(&self, policy: &Policy) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

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
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(&self, id: &PolicyId) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        sqlx::query("DELETE FROM policies WHERE id = $1")
            .bind(id.0)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id(&self, id: &PolicyId) -> AppResult<Option<Policy>> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        let row = sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT id, tenant_id, name, description, effect, subjects, resources, actions, 
                   conditions, priority, is_active, created_at, created_by, updated_at, updated_by
            FROM policies WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|r| r.into()))
    }

    async fn list_by_tenant(
        &self,
        _tenant_id: &TenantId,
        _page: u32,
        _page_size: u32,
    ) -> AppResult<(Vec<Policy>, i64)> {
        Err(AppError::internal("not implemented for tx repo"))
    }

    async fn list_active_by_tenant(&self, _tenant_id: &TenantId) -> AppResult<Vec<Policy>> {
        Err(AppError::internal("not implemented for tx repo"))
    }

    async fn find_by_subject(
        &self,
        _tenant_id: &TenantId,
        _subject: &str,
    ) -> AppResult<Vec<Policy>> {
        Err(AppError::internal("not implemented for tx repo"))
    }

    async fn find_by_resource(
        &self,
        _tenant_id: &TenantId,
        _resource: &str,
    ) -> AppResult<Vec<Policy>> {
        Err(AppError::internal("not implemented for tx repo"))
    }

    async fn exists_by_name(&self, tenant_id: &TenantId, name: &str) -> AppResult<bool> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM policies WHERE tenant_id = $1 AND name = $2)",
        )
        .bind(tenant_id.0)
        .bind(name)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.0)
    }
}

// ============ Policy 数据行映射 ============

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
                created_by: row.created_by.map(UserId::from_uuid),
                updated_at: row.updated_at,
                updated_by: row.updated_by.map(UserId::from_uuid),
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
