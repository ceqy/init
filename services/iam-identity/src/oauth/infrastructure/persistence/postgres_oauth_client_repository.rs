use async_trait::async_trait;
use cuba_common::TenantId;
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::oauth::domain::entities::{OAuthClient, OAuthClientId};
use crate::oauth::domain::repositories::OAuthClientRepository;

pub struct PostgresOAuthClientRepository {
    pool: PgPool,
}

impl PostgresOAuthClientRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OAuthClientRepository for PostgresOAuthClientRepository {
    async fn find_by_id(&self, id: &OAuthClientId, tenant_id: &TenantId) -> AppResult<Option<OAuthClient>> {
        debug!("Finding OAuth client by id: {}", id);

        let row = sqlx::query_as::<_, OAuthClientRow>(
            r#"
            SELECT id, tenant_id, name, client_secret_hash, redirect_uris, grant_types,
                   scopes, public_client, created_at, updated_at
            FROM oauth_clients
            WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find OAuth client: {}", e)))?;

        Ok(row.map(|r| r.into()))
    }

    async fn save(&self, client: &OAuthClient) -> AppResult<()> {
        debug!("Saving OAuth client: {}", client.id);

        sqlx::query(
            r#"
            INSERT INTO oauth_clients (id, tenant_id, name, client_secret_hash, redirect_uris,
                                      grant_types, scopes, public_client, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(client.id.0)
        .bind(client.tenant_id.0)
        .bind(&client.name)
        .bind(&client.client_secret_hash)
        .bind(&client.redirect_uris)
        .bind(&client.grant_types)
        .bind(&client.scopes)
        .bind(client.public_client)
        .bind(client.created_at)
        .bind(client.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save OAuth client: {}", e)))?;

        Ok(())
    }

    async fn update(&self, client: &OAuthClient) -> AppResult<()> {
        debug!("Updating OAuth client: {}", client.id);

        sqlx::query(
            r#"
            UPDATE oauth_clients
            SET name = $2, redirect_uris = $3, grant_types = $4, scopes = $5, updated_at = $6
            WHERE id = $1 AND tenant_id = $7
            "#,
        )
        .bind(client.id.0)
        .bind(&client.name)
        .bind(&client.redirect_uris)
        .bind(&client.grant_types)
        .bind(&client.scopes)
        .bind(client.updated_at)
        .bind(client.tenant_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update OAuth client: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &OAuthClientId, tenant_id: &TenantId) -> AppResult<()> {
        debug!("Deleting OAuth client: {}", id);

        sqlx::query("DELETE FROM oauth_clients WHERE id = $1 AND tenant_id = $2")
            .bind(id.0)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete OAuth client: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, id: &OAuthClientId, tenant_id: &TenantId) -> AppResult<bool> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM oauth_clients WHERE id = $1 AND tenant_id = $2)",
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to check existence: {}", e)))?;

        Ok(result.0)
    }

    async fn list_by_tenant(&self, tenant_id: &TenantId, page: i64, page_size: i64) -> AppResult<Vec<OAuthClient>> {
        let offset = (page - 1) * page_size;

        let rows = sqlx::query_as::<_, OAuthClientRow>(
            r#"
            SELECT id, tenant_id, name, client_secret_hash, redirect_uris, grant_types,
                   scopes, public_client, created_at, updated_at
            FROM oauth_clients
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id.0)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list OAuth clients: {}", e)))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn count_by_tenant(&self, tenant_id: &TenantId) -> AppResult<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM oauth_clients WHERE tenant_id = $1",
        )
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to count OAuth clients: {}", e)))?;

        Ok(count.0)
    }
}

#[derive(sqlx::FromRow)]
struct OAuthClientRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    client_secret_hash: Option<String>,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    scopes: Vec<String>,
    public_client: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<OAuthClientRow> for OAuthClient {
    fn from(row: OAuthClientRow) -> Self {
        Self {
            id: OAuthClientId::from_uuid(row.id),
            tenant_id: TenantId::from_uuid(row.tenant_id),
            name: row.name,
            client_secret_hash: row.client_secret_hash,
            redirect_uris: row.redirect_uris,
            grant_types: row.grant_types,
            scopes: row.scopes,
            public_client: row.public_client,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
