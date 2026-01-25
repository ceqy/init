//! PostgreSQL 会话 Repository 实现

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::domain::entities::{Session, SessionId};
use crate::auth::domain::repositories::SessionRepository;

pub struct PostgresSessionRepository {
    pool: PgPool,
}

impl PostgresSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for PostgresSessionRepository {
    async fn find_by_id(&self, id: &SessionId, tenant_id: &TenantId) -> AppResult<Option<Session>> {
        let row = sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT id, user_id, tenant_id, refresh_token_hash, device_info, ip_address, user_agent,
                   created_at, expires_at, last_activity_at, revoked
            FROM sessions
            WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find session: {}", e)))?;

        Ok(row.map(|r| r.into_session()))
    }

    async fn find_by_refresh_token_hash(&self, hash: &str, tenant_id: &TenantId) -> AppResult<Option<Session>> {
        let row = sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT id, user_id, tenant_id, refresh_token_hash, device_info, ip_address, user_agent,
                   created_at, expires_at, last_activity_at, revoked
            FROM sessions
            WHERE refresh_token_hash = $1 AND tenant_id = $2 AND revoked = false
            "#,
        )
        .bind(hash)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find session: {}", e)))?;

        Ok(row.map(|r| r.into_session()))
    }

    async fn find_active_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<Vec<Session>> {
        let rows = sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT id, user_id, tenant_id, refresh_token_hash, device_info, ip_address, user_agent,
                   created_at, expires_at, last_activity_at, revoked
            FROM sessions
            WHERE user_id = $1 AND tenant_id = $2 AND revoked = false AND expires_at > NOW()
            ORDER BY last_activity_at DESC
            "#,
        )
        .bind(user_id.0)
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find sessions: {}", e)))?;

        Ok(rows.into_iter().map(|r| r.into_session()).collect())
    }

    async fn save(&self, session: &Session) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, tenant_id, refresh_token_hash, device_info, ip_address, user_agent,
                                 created_at, expires_at, last_activity_at, revoked)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(session.id.0)
        .bind(session.user_id.0)
        .bind(session.tenant_id.0)
        .bind(&session.refresh_token_hash)
        .bind(&session.device_info)
        .bind(&session.ip_address)
        .bind(&session.user_agent)
        .bind(session.created_at)
        .bind(session.expires_at)
        .bind(session.last_activity_at)
        .bind(session.revoked)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save session: {}", e)))?;

        Ok(())
    }

    async fn update(&self, session: &Session) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE sessions SET
                refresh_token_hash = $2, expires_at = $3, last_activity_at = $4, revoked = $5
            WHERE id = $1
            "#,
        )
        .bind(session.id.0)
        .bind(&session.refresh_token_hash)
        .bind(session.expires_at)
        .bind(session.last_activity_at)
        .bind(session.revoked)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update session: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &SessionId, tenant_id: &TenantId) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE id = $1 AND tenant_id = $2")
            .bind(id.0)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete session: {}", e)))?;

        Ok(())
    }

    async fn revoke_all_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<()> {
        sqlx::query("UPDATE sessions SET revoked = true WHERE user_id = $1 AND tenant_id = $2")
            .bind(user_id.0)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to revoke sessions: {}", e)))?;

        Ok(())
    }

    async fn cleanup_expired(&self, tenant_id: &TenantId) -> AppResult<u64> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW() AND tenant_id = $1")
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to cleanup sessions: {}", e)))?;

        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    user_id: Uuid,
    tenant_id: Uuid,
    refresh_token_hash: String,
    device_info: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
    last_activity_at: chrono::DateTime<chrono::Utc>,
    revoked: bool,
}

impl SessionRow {
    fn into_session(self) -> Session {
        Session {
            id: SessionId(self.id),
            user_id: UserId::from_uuid(self.user_id),
            tenant_id: TenantId::from_uuid(self.tenant_id),
            refresh_token_hash: self.refresh_token_hash,
            device_info: self.device_info,
            ip_address: self.ip_address,
            user_agent: self.user_agent,
            created_at: self.created_at,
            expires_at: self.expires_at,
            last_activity_at: self.last_activity_at,
            revoked: self.revoked,
        }
    }
}
