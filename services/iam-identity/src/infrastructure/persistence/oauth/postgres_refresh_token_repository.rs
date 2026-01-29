use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::domain::oauth::{OAuthClientId, RefreshToken};
use crate::domain::repositories::oauth::RefreshTokenRepository;

pub struct PostgresRefreshTokenRepository {
    pool: PgPool,
}

impl PostgresRefreshTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RefreshTokenRepository for PostgresRefreshTokenRepository {
    async fn find_by_token(
        &self,
        token: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<RefreshToken>> {
        debug!("Finding refresh token");

        let row = sqlx::query_as::<_, RefreshTokenRow>(
            r#"
            SELECT token, tenant_id, client_id, user_id, access_token, scopes,
                   revoked, expires_at, created_at
            FROM refresh_tokens
            WHERE token = $1 AND tenant_id = $2
            "#,
        )
        .bind(token)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find refresh token: {}", e)))?;

        Ok(row.map(|r| r.into()))
    }

    async fn save(&self, token: &RefreshToken) -> AppResult<()> {
        debug!("Saving refresh token");

        let scopes_str = token.scopes.join(" ");

        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (token, tenant_id, client_id, user_id, access_token,
                                       scopes, revoked, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&token.token)
        .bind(token.tenant_id.0)
        .bind(token.client_id.0)
        .bind(token.user_id.0)
        .bind(&token.access_token)
        .bind(scopes_str)
        .bind(token.revoked)
        .bind(token.expires_at)
        .bind(token.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save refresh token: {}", e)))?;

        Ok(())
    }

    async fn update(&self, token: &RefreshToken) -> AppResult<()> {
        debug!("Updating refresh token");

        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = $2
            WHERE token = $1 AND tenant_id = $3
            "#,
        )
        .bind(&token.token)
        .bind(token.revoked)
        .bind(token.tenant_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update refresh token: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, token: &str, tenant_id: &TenantId) -> AppResult<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE token = $1 AND tenant_id = $2")
            .bind(token)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete refresh token: {}", e)))?;

        Ok(())
    }

    async fn delete_expired(&self, tenant_id: &TenantId) -> AppResult<u64> {
        let result =
            sqlx::query("DELETE FROM refresh_tokens WHERE tenant_id = $1 AND expires_at < NOW()")
                .bind(tenant_id.0)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    AppError::database(format!("Failed to delete expired tokens: {}", e))
                })?;

        Ok(result.rows_affected())
    }

    async fn delete_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<u64> {
        let result =
            sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1 AND tenant_id = $2")
                .bind(user_id.0)
                .bind(tenant_id.0)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::database(format!("Failed to delete tokens: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn delete_by_client_id(
        &self,
        client_id: &OAuthClientId,
        tenant_id: &TenantId,
    ) -> AppResult<u64> {
        let result =
            sqlx::query("DELETE FROM refresh_tokens WHERE client_id = $1 AND tenant_id = $2")
                .bind(client_id.0)
                .bind(tenant_id.0)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::database(format!("Failed to delete tokens: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn find_by_access_token(
        &self,
        access_token: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<RefreshToken>> {
        let row = sqlx::query_as::<_, RefreshTokenRow>(
            r#"
            SELECT token, tenant_id, client_id, user_id, access_token, scopes,
                   revoked, expires_at, created_at
            FROM refresh_tokens
            WHERE access_token = $1 AND tenant_id = $2
            "#,
        )
        .bind(access_token)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find refresh token: {}", e)))?;

        Ok(row.map(|r| r.into()))
    }

    async fn list_active_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<RefreshToken>> {
        let rows = sqlx::query_as::<_, RefreshTokenRow>(
            r#"
            SELECT token, tenant_id, client_id, user_id, access_token, scopes,
                   revoked, expires_at, created_at
            FROM refresh_tokens
            WHERE user_id = $1 AND tenant_id = $2 AND revoked = FALSE AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.0)
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list tokens: {}", e)))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(sqlx::FromRow)]
pub struct RefreshTokenRow {
    pub token: String,
    pub tenant_id: Uuid,
    pub client_id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub scopes: String,
    pub revoked: bool,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<RefreshTokenRow> for RefreshToken {
    fn from(row: RefreshTokenRow) -> Self {
        Self {
            token: row.token,
            tenant_id: TenantId::from_uuid(row.tenant_id),
            client_id: OAuthClientId::from_uuid(row.client_id),
            user_id: UserId::from_uuid(row.user_id),
            access_token: row.access_token,
            scopes: row
                .scopes
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            revoked: row.revoked,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}
