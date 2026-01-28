//! PostgreSQL WebAuthn 凭证仓储实现

use async_trait::async_trait;
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::domain::auth::{WebAuthnCredential, WebAuthnCredentialId};
use crate::domain::repositories::auth::WebAuthnCredentialRepository;

/// PostgreSQL WebAuthn 凭证仓储
pub struct PostgresWebAuthnCredentialRepository {
    pool: PgPool,
}

impl PostgresWebAuthnCredentialRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WebAuthnCredentialRepository for PostgresWebAuthnCredentialRepository {
    async fn save(&self, credential: &WebAuthnCredential) -> AppResult<()> {
        debug!("Saving WebAuthn credential: {}", credential.id);

        sqlx::query(
            r#"
            INSERT INTO webauthn_credentials (
                id, user_id, tenant_id, credential_id, public_key, counter,
                name, aaguid, transports, backup_eligible, backup_state,
                created_at, last_used_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(credential.id.0)
        .bind(credential.user_id)
        .bind(credential.tenant_id.0)
        .bind(&credential.credential_id)
        .bind(&credential.public_key)
        .bind(credential.counter as i64)
        .bind(&credential.name)
        .bind(credential.aaguid)
        .bind(&credential.transports)
        .bind(credential.backup_eligible)
        .bind(credential.backup_state)
        .bind(credential.created_at)
        .bind(credential.last_used_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save WebAuthn credential: {}", e)))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &WebAuthnCredentialId, tenant_id: &cuba_common::TenantId) -> AppResult<Option<WebAuthnCredential>> {
        debug!("Finding WebAuthn credential by id: {}", id);

        let row = sqlx::query_as::<_, WebAuthnCredentialRow>(
            r#"
            SELECT id, user_id, tenant_id, credential_id, public_key, counter,
                   name, aaguid, transports, backup_eligible, backup_state,
                   created_at, last_used_at
            FROM webauthn_credentials
            WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find WebAuthn credential: {}", e)))?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_credential_id(&self, credential_id: &[u8], tenant_id: &cuba_common::TenantId) -> AppResult<Option<WebAuthnCredential>> {
        debug!("Finding WebAuthn credential by credential_id");

        let row = sqlx::query_as::<_, WebAuthnCredentialRow>(
            r#"
            SELECT id, user_id, tenant_id, credential_id, public_key, counter,
                   name, aaguid, transports, backup_eligible, backup_state,
                   created_at, last_used_at
            FROM webauthn_credentials
            WHERE credential_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(credential_id)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find WebAuthn credential: {}", e)))?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_user_id(&self, user_id: &cuba_common::UserId, tenant_id: &cuba_common::TenantId) -> AppResult<Vec<WebAuthnCredential>> {
        debug!("Finding WebAuthn credentials for user: {}", user_id);

        let rows = sqlx::query_as::<_, WebAuthnCredentialRow>(
            r#"
            SELECT id, user_id, tenant_id, credential_id, public_key, counter,
                   name, aaguid, transports, backup_eligible, backup_state,
                   created_at, last_used_at
            FROM webauthn_credentials
            WHERE user_id = $1 AND tenant_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.0)
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find WebAuthn credentials: {}", e)))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn update(&self, credential: &WebAuthnCredential) -> AppResult<()> {
        debug!("Updating WebAuthn credential: {}", credential.id);

        sqlx::query(
            r#"
            UPDATE webauthn_credentials
            SET counter = $2,
                backup_eligible = $3,
                backup_state = $4,
                last_used_at = $5
            WHERE id = $1 AND tenant_id = $6
            "#,
        )
        .bind(credential.id.0)
        .bind(credential.counter as i64)
        .bind(credential.backup_eligible)
        .bind(credential.backup_state)
        .bind(credential.last_used_at)
        .bind(credential.tenant_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update WebAuthn credential: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &WebAuthnCredentialId, tenant_id: &cuba_common::TenantId) -> AppResult<()> {
        debug!("Deleting WebAuthn credential: {}", id);

        sqlx::query("DELETE FROM webauthn_credentials WHERE id = $1 AND tenant_id = $2")
            .bind(id.0)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete WebAuthn credential: {}", e)))?;

        Ok(())
    }

    async fn has_credentials(&self, user_id: &cuba_common::UserId, tenant_id: &cuba_common::TenantId) -> AppResult<bool> {
        debug!("Checking if user has WebAuthn credentials: {}", user_id);

        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM webauthn_credentials WHERE user_id = $1 AND tenant_id = $2)",
        )
        .bind(user_id.0)
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to check credentials: {}", e)))?;

        Ok(result.0)
    }
}

// 数据库行映射
#[derive(sqlx::FromRow)]
pub struct WebAuthnCredentialRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub counter: i64,
    pub name: String,
    pub aaguid: Option<Uuid>,
    pub transports: Vec<String>,
    pub backup_eligible: bool,
    pub backup_state: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<WebAuthnCredentialRow> for WebAuthnCredential {
    fn from(row: WebAuthnCredentialRow) -> Self {
        Self {
            id: WebAuthnCredentialId::from_uuid(row.id),
            user_id: row.user_id,
            tenant_id: cuba_common::TenantId::from_uuid(row.tenant_id),
            credential_id: row.credential_id,
            public_key: row.public_key,
            counter: row.counter as u32,
            name: row.name,
            aaguid: row.aaguid,
            transports: row.transports,
            backup_eligible: row.backup_eligible,
            backup_state: row.backup_state,
            created_at: row.created_at,
            last_used_at: row.last_used_at,
        }
    }
}
