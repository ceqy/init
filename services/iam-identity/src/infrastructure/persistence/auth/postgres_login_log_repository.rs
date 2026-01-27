//! PostgreSQL 登录日志仓储实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::domain::entities::{LoginLog, LoginLogId, LoginResult};
use crate::auth::domain::repositories::LoginLogRepository;

pub struct PostgresLoginLogRepository {
    pool: PgPool,
}

impl PostgresLoginLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LoginLogRepository for PostgresLoginLogRepository {
    async fn save(&self, log: &LoginLog) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO login_logs (id, user_id, tenant_id, username, ip_address, user_agent, 
                                   device_type, device_os, browser, result, failure_reason, 
                                   country, city, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(log.id.0)
        .bind(log.user_id.as_ref().map(|id| id.0))
        .bind(log.tenant_id.0)
        .bind(&log.username)
        .bind(&log.ip_address)
        .bind(&log.user_agent)
        .bind(&log.device_info.device_type)
        .bind(&log.device_info.os)
        .bind(&log.device_info.browser)
        .bind(format!("{:?}", log.result))
        .bind(log.failure_reason.as_ref().map(|r| format!("{:?}", r)))
        .bind(&log.country)
        .bind(&log.city)
        .bind(log.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save login log: {}", e)))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &LoginLogId, tenant_id: &TenantId) -> AppResult<Option<LoginLog>> {
        sqlx::query_as::<_, LoginLogRow>(
            "SELECT * FROM login_logs WHERE id = $1 AND tenant_id = $2"
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(|e| AppError::database(format!("Failed to find login log: {}", e)))
    }

    async fn find_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId, limit: i32) -> AppResult<Vec<LoginLog>> {
        sqlx::query_as::<_, LoginLogRow>(
            "SELECT * FROM login_logs WHERE user_id = $1 AND tenant_id = $2 ORDER BY created_at DESC LIMIT $3"
        )
        .bind(user_id.0)
        .bind(tenant_id.0)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|e| AppError::database(format!("Failed to find login logs: {}", e)))
    }

    async fn find_by_user_id_and_time_range(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> AppResult<Vec<LoginLog>> {
        sqlx::query_as::<_, LoginLogRow>(
            "SELECT * FROM login_logs WHERE user_id = $1 AND tenant_id = $2 AND created_at BETWEEN $3 AND $4 ORDER BY created_at DESC"
        )
        .bind(user_id.0)
        .bind(tenant_id.0)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|e| AppError::database(format!("Failed to find login logs: {}", e)))
    }

    async fn find_last_successful_login(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<Option<LoginLog>> {
        sqlx::query_as::<_, LoginLogRow>(
            "SELECT * FROM login_logs WHERE user_id = $1 AND tenant_id = $2 AND result = 'Success' ORDER BY created_at DESC LIMIT 1"
        )
        .bind(user_id.0)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(|e| AppError::database(format!("Failed to find last successful login: {}", e)))
    }

    async fn find_by_user_and_ip(&self, user_id: &UserId, tenant_id: &TenantId, ip_address: &str) -> AppResult<Vec<LoginLog>> {
        sqlx::query_as::<_, LoginLogRow>(
            "SELECT * FROM login_logs WHERE user_id = $1 AND tenant_id = $2 AND ip_address = $3 ORDER BY created_at DESC"
        )
        .bind(user_id.0)
        .bind(tenant_id.0)
        .bind(ip_address)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|e| AppError::database(format!("Failed to find login logs by IP: {}", e)))
    }

    async fn find_by_user_and_device_fingerprint(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        device_fingerprint: &str,
    ) -> AppResult<Vec<LoginLog>> {
        sqlx::query_as::<_, LoginLogRow>(
            "SELECT * FROM login_logs WHERE user_id = $1 AND tenant_id = $2 AND device_fingerprint = $3 ORDER BY created_at DESC"
        )
        .bind(user_id.0)
        .bind(tenant_id.0)
        .bind(device_fingerprint)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|e| AppError::database(format!("Failed to find login logs by device: {}", e)))
    }

    async fn count_failed_attempts(&self, user_id: &UserId, tenant_id: &TenantId, start_time: DateTime<Utc>) -> AppResult<i64> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM login_logs WHERE user_id = $1 AND tenant_id = $2 AND result = 'Failed' AND created_at >= $3"
        )
        .bind(user_id.0)
        .bind(tenant_id.0)
        .bind(start_time)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to count failed attempts: {}", e)))?;

        Ok(result.0)
    }

    async fn find_suspicious_logins(&self, tenant_id: &TenantId, start_time: DateTime<Utc>, limit: i32) -> AppResult<Vec<LoginLog>> {
        sqlx::query_as::<_, LoginLogRow>(
            "SELECT * FROM login_logs WHERE tenant_id = $1 AND created_at >= $2 AND is_suspicious = true ORDER BY created_at DESC LIMIT $3"
        )
        .bind(tenant_id.0)
        .bind(start_time)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|e| AppError::database(format!("Failed to find suspicious logins: {}", e)))
    }

    async fn list(
        &self,
        tenant_id: &TenantId,
        _user_id: Option<&UserId>,
        _result: Option<LoginResult>,
        _start_time: Option<DateTime<Utc>>,
        _end_time: Option<DateTime<Utc>>,
        page: i32,
        page_size: i32,
    ) -> AppResult<(Vec<LoginLog>, i64)> {
        let offset = (page - 1).max(0) * page_size;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM login_logs WHERE tenant_id = $1")
            .bind(tenant_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to count login logs: {}", e)))?;

        let rows = sqlx::query_as::<_, LoginLogRow>(
            "SELECT * FROM login_logs WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(tenant_id.0)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list login logs: {}", e)))?;

        Ok((rows.into_iter().map(Into::into).collect(), total.0))
    }

    async fn delete_older_than(&self, tenant_id: &TenantId, before: DateTime<Utc>) -> AppResult<u64> {
        let result = sqlx::query("DELETE FROM login_logs WHERE tenant_id = $1 AND created_at < $2")
            .bind(tenant_id.0)
            .bind(before)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete old login logs: {}", e)))?;

        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct LoginLogRow {
    id: Uuid,
    user_id: Option<Uuid>,
    tenant_id: Uuid,
    username: String,
    ip_address: String,
    user_agent: String,
    device_type: Option<String>,
    device_os: Option<String>,
    browser: Option<String>,
    result: String,
    failure_reason: Option<String>,
    country: Option<String>,
    city: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<LoginLogRow> for LoginLog {
    fn from(row: LoginLogRow) -> Self {
        use crate::auth::domain::entities::{DeviceInfo, LoginFailureReason};

        LoginLog {
            id: LoginLogId(row.id),
            user_id: row.user_id.map(UserId::from_uuid),
            tenant_id: TenantId::from_uuid(row.tenant_id),
            username: row.username,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            device_info: DeviceInfo {
                device_type: row.device_type.unwrap_or_default(),
                os: row.device_os.clone().unwrap_or_default(),
                browser: row.browser.clone().unwrap_or_default(),
                browser_version: None, // TODO: Store version
                os_version: None, // TODO: Store version
                is_mobile: false, // TODO: Detect from user agent or store
                device_fingerprint: None,
            },
            result: match row.result.as_str() {
                "Success" => LoginResult::Success,
                _ => LoginResult::Failed,
            },
            failure_reason: row.failure_reason.and_then(|r| match r.as_str() {
                "InvalidCredentials" => Some(LoginFailureReason::InvalidCredentials),
                "AccountLocked" => Some(LoginFailureReason::AccountLocked),
                "AccountDisabled" => Some(LoginFailureReason::AccountDisabled),
                _ => None,
            }),
            country: row.country,
            city: row.city,
            is_suspicious: false,
            suspicious_reason: None,
            created_at: row.created_at,
        }
    }
}
