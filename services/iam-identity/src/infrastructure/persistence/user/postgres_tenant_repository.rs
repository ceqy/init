//! PostgreSQL 租户仓储实现

use async_trait::async_trait;
use cuba_common::{AuditInfo, TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;

use crate::shared::domain::entities::{Tenant, TenantStatus};
use crate::shared::domain::repositories::TenantRepository;


pub struct PostgresTenantRepository {
    pool: PgPool,
}

impl PostgresTenantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantRepository for PostgresTenantRepository {
    async fn find_by_id(&self, id: &TenantId) -> AppResult<Option<Tenant>> {
        sqlx::query_as::<_, TenantRow>(
            "SELECT id, name, display_name, domain, settings, status, trial_ends_at, 
                    subscription_ends_at, created_at, created_by, updated_at, updated_by
             FROM tenants WHERE id = $1",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(|e| AppError::database(format!("Failed to find tenant: {}", e)))
    }

    async fn find_by_name(&self, name: &str) -> AppResult<Option<Tenant>> {
        sqlx::query_as::<_, TenantRow>(
            "SELECT id, name, display_name, domain, settings, status, trial_ends_at, 
                    subscription_ends_at, created_at, created_by, updated_at, updated_by
             FROM tenants WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(|e| AppError::database(format!("Failed to find tenant by name: {}", e)))
    }

    async fn find_by_domain(&self, domain: &str) -> AppResult<Option<Tenant>> {
        sqlx::query_as::<_, TenantRow>(
            "SELECT id, name, display_name, domain, settings, status, trial_ends_at, 
                    subscription_ends_at, created_at, created_by, updated_at, updated_by
             FROM tenants WHERE domain = $1",
        )
        .bind(domain)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(|e| AppError::database(format!("Failed to find tenant by domain: {}", e)))
    }

    async fn save(&self, tenant: &Tenant) -> AppResult<()> {
        let settings_json = serde_json::to_value(&tenant.settings)
            .map_err(|e| AppError::internal(format!("Failed to serialize settings: {}", e)))?;

        sqlx::query(
            "INSERT INTO tenants (id, name, display_name, domain, settings, status, 
                                  trial_ends_at, subscription_ends_at, created_at, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(tenant.id.0)
        .bind(&tenant.name)
        .bind(&tenant.display_name)
        .bind(&tenant.domain)
        .bind(settings_json)
        .bind(serde_json::to_string(&tenant.status).unwrap())
        .bind(tenant.trial_ends_at)
        .bind(tenant.subscription_ends_at)
        .bind(tenant.audit_info.created_at)
        .bind(tenant.audit_info.created_by.as_ref().map(|id| id.0))
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save tenant: {}", e)))?;

        Ok(())
    }

    async fn update(&self, tenant: &Tenant) -> AppResult<()> {
        let settings_json = serde_json::to_value(&tenant.settings)
            .map_err(|e| AppError::internal(format!("Failed to serialize settings: {}", e)))?;

        sqlx::query(
            "UPDATE tenants 
             SET display_name = $2, domain = $3, settings = $4, status = $5,
                 trial_ends_at = $6, subscription_ends_at = $7, updated_at = $8, updated_by = $9
             WHERE id = $1",
        )
        .bind(tenant.id.0)
        .bind(&tenant.display_name)
        .bind(&tenant.domain)
        .bind(settings_json)
        .bind(serde_json::to_string(&tenant.status).unwrap())
        .bind(tenant.trial_ends_at)
        .bind(tenant.subscription_ends_at)
        .bind(tenant.audit_info.updated_at)
        .bind(tenant.audit_info.updated_by.as_ref().map(|id| id.0))
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update tenant: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &TenantId) -> AppResult<()> {
        sqlx::query("UPDATE tenants SET status = 'Cancelled' WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete tenant: {}", e)))?;

        Ok(())
    }

    async fn exists_by_name(&self, name: &str) -> AppResult<bool> {
        let result: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM tenants WHERE name = $1)")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to check tenant name: {}", e)))?;

        Ok(result.0)
    }

    async fn exists_by_domain(&self, domain: &str) -> AppResult<bool> {
        let result: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM tenants WHERE domain = $1)")
            .bind(domain)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to check tenant domain: {}", e)))?;

        Ok(result.0)
    }

    async fn list(
        &self,
        status: Option<TenantStatus>,
        search: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> AppResult<(Vec<Tenant>, i64)> {
        let offset = (page - 1) * page_size;
        let mut conditions = vec!["1=1".to_string()];
        let mut bind_idx = 1;

        let status_str;
        let search_pattern;

        if status.is_some() {
            conditions.push(format!("status = ${}", bind_idx));
            bind_idx += 1;
        }

        if search.is_some() {
            conditions.push(format!("(name ILIKE ${} OR display_name ILIKE ${})", bind_idx, bind_idx));
            bind_idx += 1;
        }

        let query = format!(
            "SELECT id, name, display_name, domain, settings, status, trial_ends_at, 
                    subscription_ends_at, created_at, created_by, updated_at, updated_by
             FROM tenants WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            conditions.join(" AND "),
            bind_idx,
            bind_idx + 1
        );

        let mut q = sqlx::query_as::<_, TenantRow>(&query);

        if let Some(s) = &status {
            status_str = serde_json::to_string(s).unwrap();
            q = q.bind(&status_str);
        }

        if let Some(s) = search {
            search_pattern = format!("%{}%", s);
            q = q.bind(&search_pattern);
        }

        let tenants = q
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to list tenants: {}", e)))?
            .into_iter()
            .map(Into::into)
            .collect();

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenants")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to count tenants: {}", e)))?;

        Ok((tenants, total.0))
    }

    async fn count(&self) -> AppResult<i64> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenants")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to count tenants: {}", e)))?;

        Ok(result.0)
    }

    async fn find_expiring_trials(&self, days: i64) -> AppResult<Vec<Tenant>> {
        sqlx::query_as::<_, TenantRow>(
            "SELECT id, name, display_name, domain, settings, status, trial_ends_at, 
                    subscription_ends_at, created_at, created_by, updated_at, updated_by
             FROM tenants 
             WHERE status = 'Trial' 
               AND trial_ends_at IS NOT NULL 
               AND trial_ends_at <= NOW() + INTERVAL '1 day' * $1",
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|e| AppError::database(format!("Failed to find expiring trials: {}", e)))
    }

    async fn find_expiring_subscriptions(&self, days: i64) -> AppResult<Vec<Tenant>> {
        sqlx::query_as::<_, TenantRow>(
            "SELECT id, name, display_name, domain, settings, status, trial_ends_at, 
                    subscription_ends_at, created_at, created_by, updated_at, updated_by
             FROM tenants 
             WHERE status = 'Active' 
               AND subscription_ends_at IS NOT NULL 
               AND subscription_ends_at <= NOW() + INTERVAL '1 day' * $1",
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|e| AppError::database(format!("Failed to find expiring subscriptions: {}", e)))
    }
}

#[derive(sqlx::FromRow)]
struct TenantRow {
    id: uuid::Uuid,
    name: String,
    display_name: String,
    domain: Option<String>,
    settings: sqlx::types::JsonValue,
    status: String,
    trial_ends_at: Option<chrono::DateTime<chrono::Utc>>,
    subscription_ends_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    created_by: Option<uuid::Uuid>,
    updated_at: chrono::DateTime<chrono::Utc>,
    updated_by: Option<uuid::Uuid>,
}

impl From<TenantRow> for Tenant {
    fn from(row: TenantRow) -> Self {
        Self {
            id: TenantId(row.id),
            name: row.name,
            display_name: row.display_name,
            domain: row.domain,
            settings: serde_json::from_value(row.settings).unwrap_or_default(),
            status: serde_json::from_str(&format!("\"{}\"", row.status)).unwrap_or_default(),
            trial_ends_at: row.trial_ends_at,
            subscription_ends_at: row.subscription_ends_at,
            audit_info: AuditInfo {
                created_at: row.created_at,
                created_by: row.created_by.map(UserId),
                updated_at: row.updated_at,
                updated_by: row.updated_by.map(UserId),
            },
        }
    }
}
