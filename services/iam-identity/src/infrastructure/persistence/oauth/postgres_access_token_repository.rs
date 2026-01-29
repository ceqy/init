use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::domain::oauth::{AccessToken, OAuthClientId};
use crate::domain::repositories::oauth::AccessTokenRepository;

pub struct PostgresAccessTokenRepository {
    pool: PgPool,
}

impl PostgresAccessTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccessTokenRepository for PostgresAccessTokenRepository {
    async fn find_by_token(
        &self,
        token: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<AccessToken>> {
        debug!("Finding access token");

        let row = sqlx::query_as::<_, AccessTokenRow>(
            r#"
            SELECT token, tenant_id, client_id, user_id, scope, revoked,
                   expires_at, created_at
            FROM access_tokens
            WHERE token = $1 AND tenant_id = $2
            "#,
        )
        .bind(token)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find access token: {}", e)))?;

        Ok(row.map(|r| r.into()))
    }

    async fn save(&self, token: &AccessToken) -> AppResult<()> {
        debug!("Saving access token");

        let scopes_str = token.scopes.join(" ");

        sqlx::query(
            r#"
            INSERT INTO access_tokens (token, tenant_id, client_id, user_id, scope,
                                      revoked, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&token.token)
        .bind(token.tenant_id.0)
        .bind(token.client_id.0)
        .bind(token.user_id.as_ref().map(|u| u.0))
        .bind(scopes_str)
        .bind(token.revoked)
        .bind(token.expires_at)
        .bind(token.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save access token: {}", e)))?;

        Ok(())
    }

    async fn update(&self, token: &AccessToken) -> AppResult<()> {
        debug!("Updating access token");

        sqlx::query(
            r#"
            UPDATE access_tokens
            SET revoked = $2
            WHERE token = $1 AND tenant_id = $3
            "#,
        )
        .bind(&token.token)
        .bind(token.revoked)
        .bind(token.tenant_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update access token: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, token: &str, tenant_id: &TenantId) -> AppResult<()> {
        sqlx::query("DELETE FROM access_tokens WHERE token = $1 AND tenant_id = $2")
            .bind(token)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete access token: {}", e)))?;

        Ok(())
    }

    async fn delete_expired(&self, tenant_id: &TenantId) -> AppResult<u64> {
        let result =
            sqlx::query("DELETE FROM access_tokens WHERE tenant_id = $1 AND expires_at < NOW()")
                .bind(tenant_id.0)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    AppError::database(format!("Failed to delete expired tokens: {}", e))
                })?;

        Ok(result.rows_affected())
    }

    async fn delete_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<u64> {
        let result = sqlx::query("DELETE FROM access_tokens WHERE user_id = $1 AND tenant_id = $2")
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
            sqlx::query("DELETE FROM access_tokens WHERE client_id = $1 AND tenant_id = $2")
                .bind(client_id.0)
                .bind(tenant_id.0)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::database(format!("Failed to delete tokens: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn list_active_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<AccessToken>> {
        let rows = sqlx::query_as::<_, AccessTokenRow>(
            r#"
            SELECT token, tenant_id, client_id, user_id, scope, revoked,
                   expires_at, created_at
            FROM access_tokens
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
pub struct AccessTokenRow {
    pub token: String,
    pub tenant_id: Uuid,
    pub client_id: Uuid,
    pub user_id: Option<Uuid>,
    pub scope: String,
    pub revoked: bool,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<AccessTokenRow> for AccessToken {
    fn from(row: AccessTokenRow) -> Self {
        Self {
            token: row.token,
            tenant_id: TenantId::from_uuid(row.tenant_id),
            client_id: OAuthClientId::from_uuid(row.client_id),
            user_id: row.user_id.map(UserId::from_uuid),
            scopes: row
                .scope
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            revoked: row.revoked,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}
