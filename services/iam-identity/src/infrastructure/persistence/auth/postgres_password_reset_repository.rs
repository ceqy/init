//! PostgreSQL 密码重置令牌仓储实现

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::{debug, warn};

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
        debug!(token_id = %token.id, user_id = %token.user_id, "Saving password reset token");

        sqlx::query!(
            r#"
            INSERT INTO password_reset_tokens (
                id, user_id, token_hash, expires_at, used, used_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            token.id.0,
            token.user_id.0,
            token.token_hash,
            token.expires_at,
            token.used,
            token.used_at,
            token.created_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to save password reset token");
            AppError::database(format!("Failed to save password reset token: {}", e))
        })?;

        debug!(token_id = %token.id, "Password reset token saved successfully");
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &PasswordResetTokenId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PasswordResetToken>> {
        debug!(token_id = %id, tenant_id = %tenant_id, "Finding password reset token by ID");

        let row = sqlx::query!(
            r#"
            SELECT prt.id, prt.user_id, prt.token_hash, prt.expires_at, 
                   prt.used, prt.used_at, prt.created_at
            FROM password_reset_tokens prt
            INNER JOIN users u ON prt.user_id = u.id
            WHERE prt.id = $1 AND u.tenant_id = $2
            "#,
            id.0,
            tenant_id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to find password reset token");
            AppError::database(format!("Failed to find password reset token: {}", e))
        })?;

        Ok(row.map(|r| PasswordResetToken {
            id: PasswordResetTokenId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            tenant_id: tenant_id.clone(),
            token_hash: r.token_hash,
            expires_at: r.expires_at,
            used: r.used,
            used_at: r.used_at,
            created_at: r.created_at,
        }))
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PasswordResetToken>> {
        debug!(tenant_id = %tenant_id, "Finding password reset token by hash");

        let row = sqlx::query!(
            r#"
            SELECT prt.id, prt.user_id, prt.token_hash, prt.expires_at, 
                   prt.used, prt.used_at, prt.created_at
            FROM password_reset_tokens prt
            INNER JOIN users u ON prt.user_id = u.id
            WHERE prt.token_hash = $1 AND u.tenant_id = $2
            ORDER BY prt.created_at DESC
            LIMIT 1
            "#,
            token_hash,
            tenant_id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to find password reset token by hash");
            AppError::database(format!("Failed to find password reset token: {}", e))
        })?;

        Ok(row.map(|r| PasswordResetToken {
            id: PasswordResetTokenId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            tenant_id: tenant_id.clone(),
            token_hash: r.token_hash,
            expires_at: r.expires_at,
            used: r.used,
            used_at: r.used_at,
            created_at: r.created_at,
        }))
    }

    async fn update(&self, token: &PasswordResetToken) -> AppResult<()> {
        debug!(token_id = %token.id, "Updating password reset token");

        let result = sqlx::query!(
            r#"
            UPDATE password_reset_tokens
            SET used = $1, used_at = $2
            WHERE id = $3
            "#,
            token.used,
            token.used_at,
            token.id.0
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to update password reset token");
            AppError::database(format!("Failed to update password reset token: {}", e))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Password reset token not found"));
        }

        debug!(token_id = %token.id, "Password reset token updated successfully");
        Ok(())
    }

    async fn mark_as_used(
        &self,
        id: &PasswordResetTokenId,
        tenant_id: &TenantId,
    ) -> AppResult<()> {
        debug!(token_id = %id, tenant_id = %tenant_id, "Marking password reset token as used");

        let result = sqlx::query!(
            r#"
            UPDATE password_reset_tokens prt
            SET used = TRUE, used_at = NOW()
            FROM users u
            WHERE prt.id = $1 
              AND prt.user_id = u.id 
              AND u.tenant_id = $2
            "#,
            id.0,
            tenant_id.0
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to mark password reset token as used");
            AppError::database(format!("Failed to mark token as used: {}", e))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Password reset token not found"));
        }

        debug!(token_id = %id, "Password reset token marked as used");
        Ok(())
    }

    async fn delete_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<()> {
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Deleting password reset tokens for user");

        sqlx::query!(
            r#"
            DELETE FROM password_reset_tokens prt
            USING users u
            WHERE prt.user_id = $1 
              AND prt.user_id = u.id 
              AND u.tenant_id = $2
            "#,
            user_id.0,
            tenant_id.0
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to delete password reset tokens");
            AppError::database(format!("Failed to delete tokens: {}", e))
        })?;

        debug!(user_id = %user_id, "Password reset tokens deleted");
        Ok(())
    }

    async fn delete_expired(&self, tenant_id: &TenantId) -> AppResult<u64> {
        debug!(tenant_id = %tenant_id, "Deleting expired password reset tokens");

        let result = sqlx::query!(
            r#"
            DELETE FROM password_reset_tokens prt
            USING users u
            WHERE prt.expires_at < NOW() 
              AND prt.user_id = u.id 
              AND u.tenant_id = $1
            "#,
            tenant_id.0
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to delete expired tokens");
            AppError::database(format!("Failed to delete expired tokens: {}", e))
        })?;

        let deleted = result.rows_affected();
        debug!(deleted = deleted, "Expired password reset tokens deleted");
        Ok(deleted)
    }

    async fn count_unused_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<i64> {
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Counting unused password reset tokens");

        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM password_reset_tokens prt
            INNER JOIN users u ON prt.user_id = u.id
            WHERE prt.user_id = $1 
              AND u.tenant_id = $2
              AND prt.used = FALSE 
              AND prt.expires_at > NOW()
            "#,
            user_id.0,
            tenant_id.0
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to count unused tokens");
            AppError::database(format!("Failed to count unused tokens: {}", e))
        })?;

        Ok(row.count.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_save_and_find_password_reset_token(pool: PgPool) {
        let repo = PostgresPasswordResetRepository::new(pool.clone());
        let tenant_id = TenantId::from_uuid(Uuid::new_v4());
        let user_id = UserId::from_uuid(Uuid::new_v4());

        // 创建测试用户
        sqlx::query!(
            "INSERT INTO users (id, tenant_id, username, email, password_hash, status) VALUES ($1, $2, $3, $4, $5, $6)",
            user_id.0,
            tenant_id.0,
            "testuser",
            "test@example.com",
            "hash",
            "Active"
        )
        .execute(&pool)
        .await
        .unwrap();

        // 创建令牌
        let token = PasswordResetToken::new(user_id.clone(), "test_hash".to_string(), 15);
        repo.save(&token).await.unwrap();

        // 查找令牌
        let found = repo.find_by_id(&token.id, &tenant_id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().token_hash, "test_hash");
    }

    #[sqlx::test]
    async fn test_mark_as_used(pool: PgPool) {
        let repo = PostgresPasswordResetRepository::new(pool.clone());
        let tenant_id = TenantId::from_uuid(Uuid::new_v4());
        let user_id = UserId::from_uuid(Uuid::new_v4());

        // 创建测试用户
        sqlx::query!(
            "INSERT INTO users (id, tenant_id, username, email, password_hash, status) VALUES ($1, $2, $3, $4, $5, $6)",
            user_id.0,
            tenant_id.0,
            "testuser",
            "test@example.com",
            "hash",
            "Active"
        )
        .execute(&pool)
        .await
        .unwrap();

        // 创建令牌
        let token = PasswordResetToken::new(user_id, "test_hash".to_string(), 15);
        repo.save(&token).await.unwrap();

        // 标记为已使用
        repo.mark_as_used(&token.id, &tenant_id).await.unwrap();

        // 验证
        let found = repo.find_by_id(&token.id, &tenant_id).await.unwrap().unwrap();
        assert!(found.used);
        assert!(found.used_at.is_some());
    }
}
