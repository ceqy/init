//! PostgreSQL 手机验证仓储实现

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::{debug, warn};

use crate::domain::user::{
    PhoneVerification, PhoneVerificationId, PhoneVerificationStatus,
};
use crate::domain::repositories::user::PhoneVerificationRepository;

/// PostgreSQL 手机验证仓储
pub struct PostgresPhoneVerificationRepository {
    pool: PgPool,
}

/// 数据库行模型
#[derive(sqlx::FromRow)]
pub struct PhoneVerificationRow {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub phone: String,
    pub code: String,
    pub status: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl PhoneVerificationRow {
    pub fn into_verification(self) -> PhoneVerification {
        PhoneVerification {
            id: PhoneVerificationId::from_uuid(self.id),
            user_id: UserId::from_uuid(self.user_id),
            tenant_id: TenantId::from_uuid(self.tenant_id),
            phone: self.phone,
            code: self.code,
            status: match self.status.as_str() {
                "Verified" => PhoneVerificationStatus::Verified,
                "Expired" => PhoneVerificationStatus::Expired,
                _ => PhoneVerificationStatus::Pending,
            },
            expires_at: self.expires_at,
            verified_at: self.verified_at,
            created_at: self.created_at,
        }
    }
}

impl PostgresPhoneVerificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 将状态枚举转换为字符串
    fn status_to_string(status: &PhoneVerificationStatus) -> &'static str {
        match status {
            PhoneVerificationStatus::Pending => "Pending",
            PhoneVerificationStatus::Verified => "Verified",
            PhoneVerificationStatus::Expired => "Expired",
        }
    }

    /// 将字符串转换为状态枚举
    fn string_to_status(s: &str) -> PhoneVerificationStatus {
        match s {
            "Verified" => PhoneVerificationStatus::Verified,
            "Expired" => PhoneVerificationStatus::Expired,
            _ => PhoneVerificationStatus::Pending,
        }
    }
}

#[async_trait]
impl PhoneVerificationRepository for PostgresPhoneVerificationRepository {
    async fn save(&self, verification: &PhoneVerification) -> AppResult<()> {
        debug!(
            verification_id = %verification.id,
            user_id = %verification.user_id,
            phone = %verification.phone,
            "Saving phone verification"
        );

        sqlx::query!(
            r#"
            INSERT INTO phone_verifications (
                id, user_id, tenant_id, phone, code, status, 
                expires_at, verified_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            verification.id.0,
            verification.user_id.0,
            verification.tenant_id.0,
            verification.phone,
            verification.code,
            Self::status_to_string(&verification.status),
            verification.expires_at,
            verification.verified_at,
            verification.created_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to save phone verification");
            AppError::database(format!("Failed to save phone verification: {}", e))
        })?;

        debug!(verification_id = %verification.id, "Phone verification saved");
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &PhoneVerificationId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PhoneVerification>> {
        debug!(verification_id = %id, tenant_id = %tenant_id, "Finding phone verification by ID");

        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, phone, code, status,
                   expires_at, verified_at, created_at
            FROM phone_verifications
            WHERE id = $1 AND tenant_id = $2
            "#,
            id.0,
            tenant_id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to find phone verification");
            AppError::database(format!("Failed to find phone verification: {}", e))
        })?;

        Ok(row.map(|r| PhoneVerification {
            id: PhoneVerificationId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            tenant_id: TenantId::from_uuid(r.tenant_id),
            phone: r.phone,
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
    ) -> AppResult<Option<PhoneVerification>> {
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Finding latest phone verification by user ID");

        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, phone, code, status,
                   expires_at, verified_at, created_at
            FROM phone_verifications
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
            warn!(error = %e, "Failed to find latest phone verification");
            AppError::database(format!("Failed to find latest phone verification: {}", e))
        })?;

        Ok(row.map(|r| PhoneVerification {
            id: PhoneVerificationId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            tenant_id: TenantId::from_uuid(r.tenant_id),
            phone: r.phone,
            code: r.code,
            status: Self::string_to_status(&r.status),
            expires_at: r.expires_at,
            verified_at: r.verified_at,
            created_at: r.created_at,
        }))
    }

    async fn find_latest_by_phone(
        &self,
        phone: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PhoneVerification>> {
        debug!(phone = %phone, tenant_id = %tenant_id, "Finding latest phone verification by phone");

        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, phone, code, status,
                   expires_at, verified_at, created_at
            FROM phone_verifications
            WHERE phone = $1 AND tenant_id = $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            phone,
            tenant_id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to find latest phone verification by phone");
            AppError::database(format!("Failed to find latest phone verification: {}", e))
        })?;

        Ok(row.map(|r| PhoneVerification {
            id: PhoneVerificationId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            tenant_id: TenantId::from_uuid(r.tenant_id),
            phone: r.phone,
            code: r.code,
            status: Self::string_to_status(&r.status),
            expires_at: r.expires_at,
            verified_at: r.verified_at,
            created_at: r.created_at,
        }))
    }

    async fn update(&self, verification: &PhoneVerification) -> AppResult<()> {
        debug!(verification_id = %verification.id, "Updating phone verification");

        let result = sqlx::query!(
            r#"
            UPDATE phone_verifications
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
            warn!(error = %e, "Failed to update phone verification");
            AppError::database(format!("Failed to update phone verification: {}", e))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Phone verification not found"));
        }

        debug!(verification_id = %verification.id, "Phone verification updated");
        Ok(())
    }

    async fn delete_expired(&self, tenant_id: &TenantId) -> AppResult<u64> {
        debug!(tenant_id = %tenant_id, "Deleting expired phone verifications");

        let result = sqlx::query!(
            r#"
            DELETE FROM phone_verifications
            WHERE tenant_id = $1 AND expires_at < NOW()
            "#,
            tenant_id.0
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to delete expired phone verifications");
            AppError::database(format!("Failed to delete expired verifications: {}", e))
        })?;

        let deleted = result.rows_affected();
        debug!(deleted = deleted, "Expired phone verifications deleted");
        Ok(deleted)
    }

    async fn count_today_by_user(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<i64> {
        debug!(user_id = %user_id, tenant_id = %tenant_id, "Counting today's phone verifications");

        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM phone_verifications
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
            warn!(error = %e, "Failed to count today's phone verifications");
            AppError::database(format!("Failed to count verifications: {}", e))
        })?;

        Ok(row.count.unwrap_or(0))
    }
}

