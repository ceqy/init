//! PostgreSQL 用户 Repository 实现

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::user::{User, UserStatus};
use crate::domain::repositories::user::UserRepository;
use crate::domain::value_objects::{Email, HashedPassword, Username};

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &UserId, tenant_id: &TenantId) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, password_hash, display_name, phone, avatar_url,
                   tenant_id, role_ids, status, language, timezone, two_factor_enabled,
                   two_factor_secret, last_login_at, email_verified, email_verified_at,
                   phone_verified, phone_verified_at,
                   failed_login_count, locked_until, lock_reason, last_failed_login_at,
                   last_password_change_at,
                   created_at, created_by, updated_at, updated_by
            FROM users
            WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find user: {}", e)))?;

        match row {
            Some(r) => Ok(Some(r.into_user().map_err(|e| AppError::database(e))?)),
            None => Ok(None),
        }
    }

    async fn find_by_username(&self, username: &Username, tenant_id: &TenantId) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, password_hash, display_name, phone, avatar_url,
                   tenant_id, role_ids, status, language, timezone, two_factor_enabled,
                   two_factor_secret, last_login_at, email_verified, email_verified_at,
                   phone_verified, phone_verified_at,
                   failed_login_count, locked_until, lock_reason, last_failed_login_at,
                   last_password_change_at,
                   created_at, created_by, updated_at, updated_by
            FROM users
            WHERE username = $1 AND tenant_id = $2
            "#,
        )
        .bind(username.as_str())
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find user: {}", e)))?;

        match row {
            Some(r) => Ok(Some(r.into_user().map_err(|e| AppError::database(e))?)),
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &Email, tenant_id: &TenantId) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, password_hash, display_name, phone, avatar_url,
                   tenant_id, role_ids, status, language, timezone, two_factor_enabled,
                   two_factor_secret, last_login_at, email_verified, email_verified_at,
                   phone_verified, phone_verified_at,
                   failed_login_count, locked_until, lock_reason, last_failed_login_at,
                   last_password_change_at,
                   created_at, created_by, updated_at, updated_by
            FROM users
            WHERE email = $1 AND tenant_id = $2
            "#,
        )
        .bind(email.as_str())
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find user: {}", e)))?;

        match row {
            Some(r) => Ok(Some(r.into_user().map_err(|e| AppError::database(e))?)),
            None => Ok(None),
        }
    }

    async fn save(&self, user: &User) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, display_name, phone, avatar_url,
                              tenant_id, role_ids, status, language, timezone, two_factor_enabled,
                              two_factor_secret, last_login_at, email_verified, email_verified_at,
                              phone_verified, phone_verified_at,
                              failed_login_count, locked_until, lock_reason, last_failed_login_at,
                              last_password_change_at,
                              created_at, created_by, updated_at, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28)
            "#,
        )
        .bind(user.id.0)
        .bind(user.username.as_str())
        .bind(user.email.as_str())
        .bind(user.password_hash.as_str())
        .bind(&user.display_name)
        .bind(&user.phone)
        .bind(&user.avatar_url)
        .bind(user.tenant_id.0)
        .bind(&user.role_ids)
        .bind(format!("{:?}", user.status))
        .bind(&user.language)
        .bind(&user.timezone)
        .bind(user.two_factor_enabled)
        .bind(&user.two_factor_secret)
        .bind(user.last_login_at)
        .bind(user.email_verified)
        .bind(user.email_verified_at)
        .bind(user.phone_verified)
        .bind(user.phone_verified_at)
        .bind(user.failed_login_count)
        .bind(user.locked_until)
        .bind(&user.lock_reason)
        .bind(user.last_failed_login_at)
        .bind(user.last_password_change_at)
        .bind(user.audit_info.created_at)
        .bind(user.audit_info.created_by.as_ref().map(|u| u.0))
        .bind(user.audit_info.updated_at)
        .bind(user.audit_info.updated_by.as_ref().map(|u| u.0))
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save user: {}", e)))?;

        Ok(())
    }

    async fn update(&self, user: &User) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users SET
                username = $2, email = $3, password_hash = $4, display_name = $5, phone = $6,
                avatar_url = $7, role_ids = $8, status = $9, language = $10, timezone = $11,
                two_factor_enabled = $12, two_factor_secret = $13, last_login_at = $14,
                email_verified = $15, email_verified_at = $16, phone_verified = $17, phone_verified_at = $18,
                failed_login_count = $19, locked_until = $20, lock_reason = $21, last_failed_login_at = $22,
                last_password_change_at = $23, updated_at = $24, updated_by = $25
            WHERE id = $1
            "#,
        )
        .bind(user.id.0)
        .bind(user.username.as_str())
        .bind(user.email.as_str())
        .bind(user.password_hash.as_str())
        .bind(&user.display_name)
        .bind(&user.phone)
        .bind(&user.avatar_url)
        .bind(&user.role_ids)
        .bind(format!("{:?}", user.status))
        .bind(&user.language)
        .bind(&user.timezone)
        .bind(user.two_factor_enabled)
        .bind(&user.two_factor_secret)
        .bind(user.last_login_at)
        .bind(user.email_verified)
        .bind(user.email_verified_at)
        .bind(user.phone_verified)
        .bind(user.phone_verified_at)
        .bind(user.failed_login_count)
        .bind(user.locked_until)
        .bind(&user.lock_reason)
        .bind(user.last_failed_login_at)
        .bind(user.last_password_change_at)
        .bind(user.audit_info.updated_at)
        .bind(user.audit_info.updated_by.as_ref().map(|u| u.0))
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update user: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &UserId, tenant_id: &TenantId) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1 AND tenant_id = $2")
            .bind(id.0)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete user: {}", e)))?;

        Ok(())
    }

    async fn exists_by_username(&self, username: &Username, tenant_id: &TenantId) -> AppResult<bool> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND tenant_id = $2)",
        )
        .bind(username.as_str())
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to check username: {}", e)))?;

        Ok(result.0)
    }

    async fn exists_by_email(&self, email: &Email, tenant_id: &TenantId) -> AppResult<bool> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND tenant_id = $2)",
        )
        .bind(email.as_str())
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to check email: {}", e)))?;

        Ok(result.0)
    }

    async fn list(
        &self,
        tenant_id: &TenantId,
        _status: Option<&str>,
        _search: Option<&str>,
        _role_ids: &[String],
        page: i32,
        page_size: i32,
    ) -> AppResult<(Vec<User>, i64)> {
        let offset = (page - 1).max(0) * page_size;

        // 查询总数
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE tenant_id = $1"
        )
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to count users: {}", e)))?;

        // 查询数据
        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, password_hash, display_name, phone, avatar_url,
                   tenant_id, role_ids, status, language, timezone, two_factor_enabled,
                   two_factor_secret, last_login_at, email_verified, email_verified_at,
                   phone_verified, phone_verified_at,
                   failed_login_count, locked_until, lock_reason, last_failed_login_at,
                   last_password_change_at,
                   created_at, created_by, updated_at, updated_by
            FROM users
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(tenant_id.0)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list users: {}", e)))?;

        let users: Result<Vec<_>, _> = rows.into_iter().map(|r| r.into_user()).collect();
        let users = users.map_err(|e| AppError::database(e))?;

        Ok((users, total.0))
    }

    async fn count_by_tenant(&self, tenant_id: &TenantId) -> AppResult<i64> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE tenant_id = $1"
        )
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to count users: {}", e)))?;

        Ok(result.0)
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    email: String,
    password_hash: String,
    display_name: Option<String>,
    phone: Option<String>,
    avatar_url: Option<String>,
    tenant_id: Uuid,
    role_ids: Vec<String>,
    status: String,
    language: String,
    timezone: String,
    two_factor_enabled: bool,
    two_factor_secret: Option<String>,
    last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    email_verified: bool,
    email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    phone_verified: bool,
    phone_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    failed_login_count: i32,
    locked_until: Option<chrono::DateTime<chrono::Utc>>,
    lock_reason: Option<String>,
    last_failed_login_at: Option<chrono::DateTime<chrono::Utc>>,
    last_password_change_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    created_by: Option<Uuid>,
    updated_at: chrono::DateTime<chrono::Utc>,
    updated_by: Option<Uuid>,
}

impl UserRow {
    fn into_user(self) -> Result<User, String> {
        let username = Username::new(&self.username)
            .map_err(|e| format!("Invalid username in database for user {}: {}", self.id, e))?;

        let email = Email::new(&self.email)
            .map_err(|e| format!("Invalid email in database for user {}: {}", self.id, e))?;

        Ok(User {
            id: UserId::from_uuid(self.id),
            username,
            email,
            password_hash: HashedPassword::from_hash(self.password_hash),
            display_name: self.display_name,
            phone: self.phone,
            avatar_url: self.avatar_url,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            role_ids: self.role_ids,
            status: match self.status.as_str() {
                "Active" => UserStatus::Active,
                "Inactive" => UserStatus::Inactive,
                "Locked" => UserStatus::Locked,
                _ => UserStatus::PendingVerification,
            },
            language: self.language,
            timezone: self.timezone,
            two_factor_enabled: self.two_factor_enabled,
            two_factor_secret: self.two_factor_secret,
            last_login_at: self.last_login_at,
            email_verified: self.email_verified,
            email_verified_at: self.email_verified_at,
            phone_verified: self.phone_verified,
            phone_verified_at: self.phone_verified_at,
            failed_login_count: self.failed_login_count,
            locked_until: self.locked_until,
            lock_reason: self.lock_reason,
            last_failed_login_at: self.last_failed_login_at,
            last_password_change_at: self.last_password_change_at,
            audit_info: cuba_common::AuditInfo {
                created_at: self.created_at,
                created_by: self.created_by.map(UserId::from_uuid),
                updated_at: self.updated_at,
                updated_by: self.updated_by.map(UserId::from_uuid),
            },
        })
    }
}
