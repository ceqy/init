//! PostgreSQL 邮箱验证仓储实现

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::{debug, warn};


use crate::domain::user::{
    EmailVerification, EmailVerificationId, EmailVerificationStatus,
};
use crate::domain::repositories::user::EmailVerificationRepository;

/// PostgreSQL 邮箱验证仓储
pub struct PostgresEmailVerificationRepository {
    pool: PgPool,
}

impl PostgresEmailVerificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 将状态枚举转换为字符串
    fn status_to_string(status: &EmailVerificationStatus) -> &'static str {
        match status {
            EmailVerificationStatus::Pending => "Pending",
            EmailVerificationStatus::Verified => "Verified",
            EmailVerificationStatus::Expired => "Expired",
        }
    }

    /// 将字符串转换为状态枚举
    fn string_to_status(s: &str) -> EmailVerificationStatus {
        match s {
            "Verified" => EmailVerificationStatus::Verified,
            "Expired" => EmailVerificationStatus::Expired,
            _ => EmailVerificationStatus::Pending,
        }
    }
}

#[async_trait]
impl EmailVerificationRepository for PostgresEmailVerificationRepository {
    async fn save(&self, verification: &EmailVerification) -> AppResult<()> {
        debug!(
            verification_id = %verification.id,
            user_id = %verification.user_id,
            email = %verification.email,
            "Saving email verification"
        );

        sqlx::query!(
            r#"
            INSERT INTO email_verifications (
                id, user_id, tenant_id, email, code, status, 
                expires_at, verified_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            verification.id.0,
            verification.user_id.0,
            verification.tenant_id.0,
            verification.email,
            verification.code,
            Self::status_to_string(&verification.status),
            verification.expires_at,
            verification.verified_at,
            verification.created_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to save email verification");
            AppError::database(format!("Failed to save email verification: {}", e))
        })?;

        debug!(verification_id = %verification.id, "Email verification saved");
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &EmailVerificationId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<EmailVerification>> {
        debug!(verification_id = %id, tenant_id = %tenant_id, "Finding email verification by ID");

        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, email, code, status,
                   expires_at, verified_at, created_at
            FROM email_verifications
            WHERE id = $1 AND tenant_id = $2
            "#,
            id.0,
            tenant_id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to find email verification");
            AppError::database(format!("Failed to find email verification: {}", e))
        })?;

        Ok(row.map(|r| EmailVerification {
            id: EmailVerificationId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            tenant_id: TenantId::from_uuid(r.tenant_id),
            email: r.email,
            code: r.code,
            status: Self::string_to_status(&r.status),
            expires_at: r.expires_at,
            verified_at: r.verified_at,
            created_at: r.created_at,
        }))
    }

    async fn find_latest_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<EmailVerification>> {
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Finding latest email verification by user ID");

        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, email, code, status,
                   expires_at, verified_at, created_at
            FROM email_verifications
            WHERE user_id = $1 AND tenant_id = $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            user_id.0,
            tenant_id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to find latest email verification");
            AppError::database(format!("Failed to find latest email verification: {}", e))
        })?;

        Ok(row.map(|r| EmailVerification {
            id: EmailVerificationId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            tenant_id: TenantId::from_uuid(r.tenant_id),
            email: r.email,
            code: r.code,
            status: Self::string_to_status(&r.status),
            expires_at: r.expires_at,
            verified_at: r.verified_at,
            created_at: r.created_at,
        }))
    }

    async fn find_latest_by_email(
        &self,
        email: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<EmailVerification>> {
        debug!(email = %email, tenant_id = %tenant_id, "Finding latest email verification by email");

        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, email, code, status,
                   expires_at, verified_at, created_at
            FROM email_verifications
            WHERE email = $1 AND tenant_id = $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            email,
            tenant_id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to find latest email verification by email");
            AppError::database(format!("Failed to find latest email verification: {}", e))
        })?;

        Ok(row.map(|r| EmailVerification {
            id: EmailVerificationId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            tenant_id: TenantId::from_uuid(r.tenant_id),
            email: r.email,
            code: r.code,
            status: Self::string_to_status(&r.status),
            expires_at: r.expires_at,
            verified_at: r.verified_at,
            created_at: r.created_at,
        }))
    }

    async fn update(&self, verification: &EmailVerification) -> AppResult<()> {
        debug!(verification_id = %verification.id, "Updating email verification");

        let result = sqlx::query!(
            r#"
            UPDATE email_verifications
            SET status = $1, verified_at = $2
            WHERE id = $3
            "#,
            Self::status_to_string(&verification.status),
            verification.verified_at,
            verification.id.0
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to update email verification");
            AppError::database(format!("Failed to update email verification: {}", e))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Email verification not found"));
        }

        debug!(verification_id = %verification.id, "Email verification updated");
        Ok(())
    }

    async fn delete_expired(&self, tenant_id: &TenantId) -> AppResult<u64> {
        debug!(tenant_id = %tenant_id, "Deleting expired email verifications");

        let result = sqlx::query!(
            r#"
            DELETE FROM email_verifications
            WHERE tenant_id = $1 AND expires_at < NOW()
            "#,
            tenant_id.0
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to delete expired email verifications");
            AppError::database(format!("Failed to delete expired verifications: {}", e))
        })?;

        let deleted = result.rows_affected();
        debug!(deleted = deleted, "Expired email verifications deleted");
        Ok(deleted)
    }

    async fn count_today_by_user(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<i64> {
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Counting today's email verifications");

        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM email_verifications
            WHERE user_id = $1 
              AND tenant_id = $2
              AND created_at >= CURRENT_DATE
            "#,
            user_id.0,
            tenant_id.0
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to count today's email verifications");
            AppError::database(format!("Failed to count verifications: {}", e))
        })?;

        Ok(row.count.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_save_and_find_email_verification(pool: PgPool) {
        let repo = PostgresEmailVerificationRepository::new(pool.clone());
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

        // 创建验证记录
        let verification = EmailVerification::new(
            user_id.clone(),
            tenant_id.clone(),
            "test@example.com".to_string(),
        );
        repo.save(&verification).await.unwrap();

        // 查找验证记录
        let found = repo.find_by_id(&verification.id, &tenant_id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().email, "test@example.com");
    }

    #[sqlx::test]
    async fn test_find_latest_by_user_id(pool: PgPool) {
        let repo = PostgresEmailVerificationRepository::new(pool.clone());
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

        // 创建多个验证记录
        for _ in 0..3 {
            let verification = EmailVerification::new(
                user_id.clone(),
                tenant_id.clone(),
                "test@example.com".to_string(),
            );
            repo.save(&verification).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // 查找最新的验证记录
        let latest = repo
            .find_latest_by_user_id(&user_id, &tenant_id)
            .await
            .unwrap();
        assert!(latest.is_some());
    }

    #[sqlx::test]
    async fn test_count_today_by_user(pool: PgPool) {
        let repo = PostgresEmailVerificationRepository::new(pool.clone());
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

        // 创建验证记录
        for _ in 0..5 {
            let verification = EmailVerification::new(
                user_id.clone(),
                tenant_id.clone(),
                "test@example.com".to_string(),
            );
            repo.save(&verification).await.unwrap();
        }

        // 统计今天的验证记录
        let count = repo.count_today_by_user(&user_id, &tenant_id).await.unwrap();
        assert_eq!(count, 5);
    }
}
