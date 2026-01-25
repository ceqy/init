//! PostgreSQL 用户 Repository 实现

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::domain::entities::{User, UserStatus};
use crate::shared::domain::repositories::UserRepository;
use crate::shared::domain::value_objects::{Email, HashedPassword, Username};

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
    async fn find_by_id(&self, id: &UserId) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, password_hash, display_name, phone, avatar_url,
                   tenant_id, role_ids, status, language, timezone, two_factor_enabled,
                   two_factor_secret, last_login_at, created_at, created_by, updated_at, updated_by
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find user: {}", e)))?;

        Ok(row.map(|r| r.into_user()))
    }

    async fn find_by_username(&self, username: &Username, tenant_id: &TenantId) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, password_hash, display_name, phone, avatar_url,
                   tenant_id, role_ids, status, language, timezone, two_factor_enabled,
                   two_factor_secret, last_login_at, created_at, created_by, updated_at, updated_by
            FROM users
            WHERE username = $1 AND tenant_id = $2
            "#,
        )
        .bind(username.as_str())
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find user: {}", e)))?;

        Ok(row.map(|r| r.into_user()))
    }

    async fn find_by_email(&self, email: &Email, tenant_id: &TenantId) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, password_hash, display_name, phone, avatar_url,
                   tenant_id, role_ids, status, language, timezone, two_factor_enabled,
                   two_factor_secret, last_login_at, created_at, created_by, updated_at, updated_by
            FROM users
            WHERE email = $1 AND tenant_id = $2
            "#,
        )
        .bind(email.as_str())
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find user: {}", e)))?;

        Ok(row.map(|r| r.into_user()))
    }

    async fn save(&self, user: &User) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, display_name, phone, avatar_url,
                              tenant_id, role_ids, status, language, timezone, two_factor_enabled,
                              two_factor_secret, last_login_at, created_at, created_by, updated_at, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
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
                updated_at = $15, updated_by = $16
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
        .bind(user.audit_info.updated_at)
        .bind(user.audit_info.updated_by.as_ref().map(|u| u.0))
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update user: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &UserId) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id.0)
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
    created_at: chrono::DateTime<chrono::Utc>,
    created_by: Option<Uuid>,
    updated_at: chrono::DateTime<chrono::Utc>,
    updated_by: Option<Uuid>,
}

impl UserRow {
    fn into_user(self) -> User {
        User {
            id: UserId::from_uuid(self.id),
            username: Username::new(&self.username).unwrap(),
            email: Email::new(&self.email).unwrap(),
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
            audit_info: cuba_common::AuditInfo {
                created_at: self.created_at,
                created_by: self.created_by.map(UserId::from_uuid),
                updated_at: self.updated_at,
                updated_by: self.updated_by.map(UserId::from_uuid),
            },
        }
    }
}
