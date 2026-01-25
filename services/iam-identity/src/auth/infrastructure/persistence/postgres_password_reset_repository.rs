//! PostgreSQL 密码重置令牌仓储实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cuba_common::UserId;
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::auth::domain::entities::{PasswordResetToken, PasswordResetTokenId};
use crate::auth::domain::repositories::PasswordResetRepository;

/// PostgreSQL 密码重置令牌仓储
pub struct PostgresPasswordResetRepository {
    pool: PgPool,
}

impl PostgresPasswordResetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PasswordResetRepository for PostgresPasswordResetRepository {
    async fn save(&self, token: &PasswordResetToken) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (
                id, user_id, token_hash, expires_at, used, used_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(token.id.0)
        .bind(token.user_id.0)
        .bind(&token.token_hash)
        .bind(token.expires_at)
        .bind(token.used)
        .bind(token.used_at)
        .bind(token.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::database(format!("Failed to save password reset token: {}", e))
        })?;

        debug!(token_id = %token.id, user_id = %token.user_id, "Password reset token saved");
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &PasswordResetTokenId,
    ) -> AppResult<Option<PasswordResetToken>> {
        let row = sqlx::query_as::<_, PasswordResetTokenRow>(
            r#"
            SELECT id, user_id, token_hash, expires_at, used, used_at, created_at
            FROM password_reset_tokens
            WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::database(format!("Failed to find password reset token: {}", e))
        })?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> AppResult<Option<PasswordResetToken>> {
        let row = sqlx::query_as::<_, PasswordResetTokenRow>(
            r#"
            SELECT id, user_id, token_hash, expires_at, used, used_at, created_at
            FROM password_reset_tokens
            WHERE token_hash = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::database(format!("Failed to find password reset token by hash: {}", e))
        })?;

        Ok(row.map(|r| r.into()))
    }

    async fn update(&self, token: &PasswordResetToken) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used = $2, used_at = $3
            WHERE id = $1
            "#,
        )
        .bind(token.id.0)
        .bind(token.used)
        .bind(token.used_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::database(format!("Failed to update password reset token: {}", e))
        })?;

        debug!(token_id = %token.id, "Password reset token updated");
        Ok(())
    }

    async fn mark_as_used(&self, id: &PasswordResetTokenId) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used = TRUE, used_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::database(format!("Failed to mark password reset token as used: {}", e))
        })?;

        debug!(token_id = %id, "Password reset token marked as used");
        Ok(())
    }

    async fn delete_by_user_id(&self, user_id: &UserId) -> AppResult<()> {
        sqlx::query(
            r#"
            DELETE FROM password_reset_tokens
            WHERE user_id = $1
            "#,
        )
        .bind(user_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::database(format!(
                "Failed to delete password reset tokens for user: {}",
                e
            ))
        })?;

        debug!(user_id = %user_id, "Password reset tokens deleted for user");
        Ok(())
    }

    async fn delete_expired(&self) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM password_reset_tokens
            WHERE expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::database(format!("Failed to delete expired password reset tokens: {}", e))
        })?;

        let count = result.rows_affected();
        debug!(count = %count, "Expired password reset tokens deleted");
        Ok(count)
    }

    async fn count_unused_by_user_id(&self, user_id: &UserId) -> AppResult<i64> {
        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM password_reset_tokens
            WHERE user_id = $1 AND used = FALSE AND expires_at > NOW()
            "#,
        )
        .bind(user_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            AppError::database(format!(
                "Failed to count unused password reset tokens: {}",
                e
            ))
        })?;

        Ok(row.0)
    }
}

// 数据库行映射
#[derive(sqlx::FromRow)]
struct PasswordResetTokenRow {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    expires_at: DateTime<Utc>,
    used: bool,
    used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<PasswordResetTokenRow> for PasswordResetToken {
    fn from(row: PasswordResetTokenRow) -> Self {
        Self {
            id: PasswordResetTokenId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            token_hash: row.token_hash,
            expires_at: row.expires_at,
            used: row.used,
            used_at: row.used_at,
            created_at: row.created_at,
        }
    }
}
