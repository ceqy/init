use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::domain::oauth::{AuthorizationCode, OAuthClientId};
use crate::domain::repositories::oauth::AuthorizationCodeRepository;

pub struct PostgresAuthorizationCodeRepository {
    pool: PgPool,
}

impl PostgresAuthorizationCodeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthorizationCodeRepository for PostgresAuthorizationCodeRepository {
    async fn find_by_code(
        &self,
        code: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<AuthorizationCode>> {
        debug!("Finding authorization code");

        let row = sqlx::query_as::<_, AuthorizationCodeRow>(
            r#"
            SELECT code, tenant_id, client_id, user_id, redirect_uri, scopes,
                   code_challenge, code_challenge_method, used, expires_at, created_at
            FROM authorization_codes
            WHERE code = $1 AND tenant_id = $2
            "#,
        )
        .bind(code)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find authorization code: {}", e)))?;

        Ok(row.map(|r| r.into()))
    }

    async fn save(&self, authorization_code: &AuthorizationCode) -> AppResult<()> {
        debug!("Saving authorization code");

        // Store scopes as space-separated string to match database column type
        let scopes_str = authorization_code.scopes.join(" ");

        sqlx::query(
            r#"
            INSERT INTO authorization_codes (code, tenant_id, client_id, user_id, redirect_uri,
                                            scopes, code_challenge, code_challenge_method, used,
                                            expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(&authorization_code.code)
        .bind(authorization_code.tenant_id.0)
        .bind(authorization_code.client_id.0)
        .bind(authorization_code.user_id.0)
        .bind(&authorization_code.redirect_uri)
        .bind(&scopes_str)
        .bind(&authorization_code.code_challenge)
        .bind(&authorization_code.code_challenge_method)
        .bind(authorization_code.used)
        .bind(authorization_code.expires_at)
        .bind(authorization_code.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save authorization code: {}", e)))?;

        Ok(())
    }

    async fn update(&self, authorization_code: &AuthorizationCode) -> AppResult<()> {
        debug!("Updating authorization code");

        sqlx::query(
            r#"
            UPDATE authorization_codes
            SET used = $2
            WHERE code = $1 AND tenant_id = $3
            "#,
        )
        .bind(&authorization_code.code)
        .bind(authorization_code.used)
        .bind(authorization_code.tenant_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update authorization code: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, code: &str, tenant_id: &TenantId) -> AppResult<()> {
        sqlx::query("DELETE FROM authorization_codes WHERE code = $1 AND tenant_id = $2")
            .bind(code)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::database(format!("Failed to delete authorization code: {}", e))
            })?;

        Ok(())
    }

    async fn delete_expired(&self, tenant_id: &TenantId) -> AppResult<u64> {
        let result = sqlx::query(
            "DELETE FROM authorization_codes WHERE tenant_id = $1 AND expires_at < NOW()",
        )
        .bind(tenant_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to delete expired codes: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn delete_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<u64> {
        let result =
            sqlx::query("DELETE FROM authorization_codes WHERE user_id = $1 AND tenant_id = $2")
                .bind(user_id.0)
                .bind(tenant_id.0)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::database(format!("Failed to delete codes: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn delete_by_client_id(
        &self,
        client_id: &OAuthClientId,
        tenant_id: &TenantId,
    ) -> AppResult<u64> {
        let result =
            sqlx::query("DELETE FROM authorization_codes WHERE client_id = $1 AND tenant_id = $2")
                .bind(client_id.0)
                .bind(tenant_id.0)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::database(format!("Failed to delete codes: {}", e)))?;

        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
pub struct AuthorizationCodeRow {
    pub code: String,
    pub tenant_id: Uuid,
    pub client_id: Uuid,
    pub user_id: Uuid,
    pub redirect_uri: String,
    pub scopes: String,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub used: bool,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<AuthorizationCodeRow> for AuthorizationCode {
    fn from(row: AuthorizationCodeRow) -> Self {
        Self {
            code: row.code,
            tenant_id: TenantId::from_uuid(row.tenant_id),
            client_id: OAuthClientId::from_uuid(row.client_id),
            user_id: UserId::from_uuid(row.user_id),
            redirect_uri: row.redirect_uri,
            scopes: row
                .scopes
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            code_challenge: row.code_challenge,
            code_challenge_method: row.code_challenge_method,
            used: row.used,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}
