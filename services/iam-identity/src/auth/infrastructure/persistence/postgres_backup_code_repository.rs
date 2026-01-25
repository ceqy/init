//! PostgreSQL 备份码仓储实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cuba_common::UserId;
use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::domain::entities::{BackupCode, BackupCodeId};
use crate::auth::domain::repositories::BackupCodeRepository;

pub struct PostgresBackupCodeRepository {
    pool: PgPool,
}

impl PostgresBackupCodeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct BackupCodeRow {
    id: Uuid,
    user_id: Uuid,
    code_hash: String,
    used: bool,
    used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl BackupCodeRow {
    fn into_backup_code(self) -> BackupCode {
        BackupCode {
            id: BackupCodeId(self.id),
            user_id: UserId::from_uuid(self.user_id),
            code_hash: self.code_hash,
            used: self.used,
            used_at: self.used_at,
            created_at: self.created_at,
        }
    }
}

#[async_trait]
impl BackupCodeRepository for PostgresBackupCodeRepository {
    async fn save(&self, backup_code: &BackupCode) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO backup_codes (id, user_id, code_hash, used, used_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(backup_code.id.0)
        .bind(backup_code.user_id.0)
        .bind(&backup_code.code_hash)
        .bind(backup_code.used)
        .bind(backup_code.used_at)
        .bind(backup_code.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to save backup code: {}", e)))?;

        Ok(())
    }

    async fn save_batch(&self, backup_codes: &[BackupCode]) -> AppResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        for code in backup_codes {
            sqlx::query(
                r#"
                INSERT INTO backup_codes (id, user_id, code_hash, used, used_at, created_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(code.id.0)
            .bind(code.user_id.0)
            .bind(&code.code_hash)
            .bind(code.used)
            .bind(code.used_at)
            .bind(code.created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to save backup code: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &BackupCodeId) -> AppResult<Option<BackupCode>> {
        let row = sqlx::query_as::<_, BackupCodeRow>(
            r#"
            SELECT id, user_id, code_hash, used, used_at, created_at
            FROM backup_codes
            WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find backup code: {}", e)))?;

        Ok(row.map(|r| r.into_backup_code()))
    }

    async fn find_available_by_user_id(&self, user_id: &UserId) -> AppResult<Vec<BackupCode>> {
        let rows = sqlx::query_as::<_, BackupCodeRow>(
            r#"
            SELECT id, user_id, code_hash, used, used_at, created_at
            FROM backup_codes
            WHERE user_id = $1 AND used = FALSE
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find backup codes: {}", e)))?;

        Ok(rows.into_iter().map(|r| r.into_backup_code()).collect())
    }

    async fn update(&self, backup_code: &BackupCode) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE backup_codes
            SET used = $1, used_at = $2
            WHERE id = $3
            "#,
        )
        .bind(backup_code.used)
        .bind(backup_code.used_at)
        .bind(backup_code.id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update backup code: {}", e)))?;

        Ok(())
    }

    async fn delete_by_user_id(&self, user_id: &UserId) -> AppResult<()> {
        sqlx::query(
            r#"
            DELETE FROM backup_codes
            WHERE user_id = $1
            "#,
        )
        .bind(user_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to delete backup codes: {}", e)))?;

        Ok(())
    }

    async fn count_available_by_user_id(&self, user_id: &UserId) -> AppResult<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM backup_codes
            WHERE user_id = $1 AND used = FALSE
            "#,
        )
        .bind(user_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to count backup codes: {}", e)))?;

        Ok(count.0)
    }
}
