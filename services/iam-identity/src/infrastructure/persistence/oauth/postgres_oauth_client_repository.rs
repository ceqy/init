use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::domain::oauth::{OAuthClient, OAuthClientId};
use crate::domain::repositories::oauth::OAuthClientRepository;

pub struct PostgresOAuthClientRepository {
    pool: PgPool,
}

#[derive(sqlx::FromRow)]
pub struct OAuthClientRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub client_secret_hash: Option<String>,
    pub client_type: String, // Stored as string
    pub grant_types: Vec<String>,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,

    pub public_client: bool,
    pub access_token_lifetime: i32,
    pub refresh_token_lifetime: i32,
    pub require_pkce: bool,
    pub require_consent: bool,
    pub is_active: bool,
    pub logo_url: Option<String>,
    pub homepage_url: Option<String>,
    pub privacy_policy_url: Option<String>,
    pub terms_of_service_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<OAuthClientRow> for OAuthClient {
    fn from(row: OAuthClientRow) -> Self {
        use crate::domain::oauth::{GrantType, OAuthClientId, OAuthClientType};

        let client_type = match row.client_type.as_str() {
            "Confidential" => OAuthClientType::Confidential,
            "Public" => OAuthClientType::Public,
            _ => OAuthClientType::Confidential, // Default or Error
        };

        let grant_types = row
            .grant_types
            .iter()
            .map(|s| {
                match s.as_str() {
                    "authorization_code" => GrantType::AuthorizationCode,
                    "client_credentials" => GrantType::ClientCredentials,
                    "refresh_token" => GrantType::RefreshToken,
                    "implicit" => GrantType::Implicit,
                    "password" => GrantType::Password,
                    _ => GrantType::AuthorizationCode, // Fallback
                }
            })
            .collect();

        Self {
            id: OAuthClientId::from_uuid(row.id),
            tenant_id: TenantId::from_uuid(row.tenant_id),
            owner_id: UserId::from_uuid(row.owner_id),
            name: row.name,
            description: row.description,
            client_secret_hash: row.client_secret_hash,
            client_type,
            grant_types,
            redirect_uris: row.redirect_uris,
            allowed_scopes: row.allowed_scopes.clone(),
            scopes: row.allowed_scopes,
            public_client: row.public_client,
            access_token_lifetime: row.access_token_lifetime as i64,
            refresh_token_lifetime: row.refresh_token_lifetime as i64,
            require_pkce: row.require_pkce,
            require_consent: row.require_consent,
            is_active: row.is_active,
            logo_url: row.logo_url,
            homepage_url: row.homepage_url,
            privacy_policy_url: row.privacy_policy_url,
            terms_of_service_url: row.terms_of_service_url,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl PostgresOAuthClientRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OAuthClientRepository for PostgresOAuthClientRepository {
    async fn find_by_id(
        &self,
        id: &OAuthClientId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<OAuthClient>> {
        debug!("Finding OAuth client by id: {}", id);

        let row = sqlx::query_as::<_, OAuthClientRow>(
            r#"
            SELECT id, tenant_id, owner_id, name, description, client_secret_hash, client_type,
                   grant_types, redirect_uris, allowed_scopes, public_client,
                   access_token_lifetime, refresh_token_lifetime, require_pkce, require_consent,
                   is_active, logo_url, homepage_url, privacy_policy_url, terms_of_service_url,
                   created_at, updated_at
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

        let grant_types: Vec<String> = client.grant_types.iter().map(|g| g.to_string()).collect();
        let client_type = format!("{:?}", client.client_type);

        sqlx::query(
            r#"
            INSERT INTO oauth_clients (
                id, tenant_id, owner_id, name, description, client_secret_hash, client_type,
                grant_types, redirect_uris, allowed_scopes, public_client,
                access_token_lifetime, refresh_token_lifetime, require_pkce, require_consent,
                is_active, logo_url, homepage_url, privacy_policy_url, terms_of_service_url,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            "#,
        )
        .bind(client.id.0)
        .bind(client.tenant_id.0)
        .bind(client.owner_id.0)
        .bind(&client.name)
        .bind(&client.description)
        .bind(&client.client_secret_hash)
        .bind(&client_type)
        .bind(&grant_types)
        .bind(&client.redirect_uris)
        .bind(&client.allowed_scopes)

        .bind(client.public_client)
        .bind(client.access_token_lifetime)
        .bind(client.refresh_token_lifetime)
        .bind(client.require_pkce)
        .bind(client.require_consent)
        .bind(client.is_active)
        .bind(&client.logo_url)
        .bind(&client.homepage_url)
        .bind(&client.privacy_policy_url)
        .bind(&client.terms_of_service_url)
        .bind(client.created_at)
        .bind(client.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save OAuth client: {}", e)))?;

        Ok(())
    }

    async fn update(&self, client: &OAuthClient) -> AppResult<()> {
        debug!("Updating OAuth client: {}", client.id);

        let grant_types: Vec<String> = client.grant_types.iter().map(|g| g.to_string()).collect();

        sqlx::query(
            r#"
            UPDATE oauth_clients
            SET name = $2, description = $3, redirect_uris = $4, grant_types = $5,
                allowed_scopes = $6, is_active = $7, logo_url = $8,
                homepage_url = $9, privacy_policy_url = $10, terms_of_service_url = $11, updated_at = $12
            WHERE id = $1 AND tenant_id = $13
            "#,
        )
        .bind(client.id.0)
        .bind(&client.name)
        .bind(&client.description)
        .bind(&client.redirect_uris)
        .bind(&grant_types)
        .bind(&client.allowed_scopes)

        .bind(client.is_active)
        .bind(&client.logo_url)
        .bind(&client.homepage_url)
        .bind(&client.privacy_policy_url)
        .bind(&client.terms_of_service_url)
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

    async fn list_by_tenant(
        &self,
        tenant_id: &TenantId,
        page: i64,
        page_size: i64,
    ) -> AppResult<Vec<OAuthClient>> {
        let offset = (page - 1) * page_size;

        let rows = sqlx::query_as::<_, OAuthClientRow>(
            r#"
            SELECT id, tenant_id, owner_id, name, description, client_secret_hash, client_type,
                   grant_types, redirect_uris, allowed_scopes, public_client,
                   access_token_lifetime, refresh_token_lifetime, require_pkce, require_consent,
                   is_active, logo_url, homepage_url, privacy_policy_url, terms_of_service_url,
                   created_at, updated_at
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
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM oauth_clients WHERE tenant_id = $1")
                .bind(tenant_id.0)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::database(format!("Failed to count OAuth clients: {}", e)))?;

        Ok(count.0)
    }
}
