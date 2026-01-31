//! PostgreSQL 迁移管理模块
//!
//! 提供数据库迁移的管理功能

use cuba_errors::{AppError, AppResult};
use sqlx::PgPool;
use tracing::{info, warn};

/// 迁移记录
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MigrationRecord {
    /// 迁移版本
    pub version: i64,
    /// 迁移名称
    pub name: String,
    /// 应用时间
    pub applied_at: chrono::DateTime<chrono::Utc>,
    /// 校验和
    pub checksum: String,
}

/// 迁移定义
#[derive(Debug, Clone)]
pub struct Migration {
    /// 版本号
    pub version: i64,
    /// 名称
    pub name: String,
    /// 升级 SQL
    pub up_sql: String,
    /// 降级 SQL（可选）
    pub down_sql: Option<String>,
    /// 校验和
    pub checksum: String,
}

impl Migration {
    /// 创建新的迁移
    pub fn new(version: i64, name: impl Into<String>, up_sql: impl Into<String>) -> Self {
        let up_sql = up_sql.into();
        let checksum = Self::calculate_checksum(&up_sql);
        Self {
            version,
            name: name.into(),
            up_sql,
            down_sql: None,
            checksum,
        }
    }

    /// 设置降级 SQL
    pub fn with_down(mut self, down_sql: impl Into<String>) -> Self {
        self.down_sql = Some(down_sql.into());
        self
    }

    /// 计算校验和
    fn calculate_checksum(sql: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// 迁移管理器
pub struct MigrationManager {
    pool: PgPool,
    table_name: String,
    schema: Option<String>,
}

impl MigrationManager {
    /// 创建新的迁移管理器
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            table_name: "_migrations".to_string(),
            schema: None,
        }
    }

    /// 设置迁移表名
    pub fn with_table_name(mut self, name: impl Into<String>) -> Self {
        self.table_name = name.into();
        self
    }

    /// 设置 Schema
    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    /// 获取完整表名
    fn full_table_name(&self) -> String {
        match &self.schema {
            Some(schema) => format!("{}.{}", schema, self.table_name),
            None => self.table_name.clone(),
        }
    }

    /// 初始化迁移表
    pub async fn init(&self) -> AppResult<()> {
        let table = self.full_table_name();

        // 创建 schema（如果指定）
        if let Some(ref schema) = self.schema {
            sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {}", schema))
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::database(format!("Failed to create schema: {}", e)))?;
        }

        // 创建迁移表
        let create_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                version BIGINT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                checksum VARCHAR(64) NOT NULL
            )
            "#,
            table
        );

        sqlx::query(&create_sql)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to create migration table: {}", e)))?;

        info!(table = %table, "Migration table initialized");
        Ok(())
    }

    /// 获取已应用的迁移
    pub async fn get_applied_migrations(&self) -> AppResult<Vec<MigrationRecord>> {
        let table = self.full_table_name();
        let sql = format!(
            "SELECT version, name, applied_at, checksum FROM {} ORDER BY version ASC",
            table
        );

        let records = sqlx::query_as::<_, MigrationRecord>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to get migrations: {}", e)))?;

        Ok(records)
    }

    /// 获取当前版本
    pub async fn current_version(&self) -> AppResult<Option<i64>> {
        let table = self.full_table_name();
        let sql = format!("SELECT MAX(version) FROM {}", table);

        let row: Option<(Option<i64>,)> = sqlx::query_as(&sql)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to get current version: {}", e)))?;

        Ok(row.and_then(|(v,)| v))
    }

    /// 应用单个迁移
    pub async fn apply(&self, migration: &Migration) -> AppResult<()> {
        let table = self.full_table_name();

        // 开始事务
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        // 检查是否已应用
        let check_sql = format!("SELECT version FROM {} WHERE version = $1", table);
        let existing: Option<(i64,)> = sqlx::query_as(&check_sql)
            .bind(migration.version)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to check migration: {}", e)))?;

        if existing.is_some() {
            warn!(
                version = migration.version,
                name = %migration.name,
                "Migration already applied, skipping"
            );
            return Ok(());
        }

        // 执行迁移 SQL
        sqlx::query(&migration.up_sql)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                AppError::database(format!(
                    "Failed to apply migration {}: {}",
                    migration.version, e
                ))
            })?;

        // 记录迁移
        let insert_sql = format!(
            "INSERT INTO {} (version, name, checksum) VALUES ($1, $2, $3)",
            table
        );
        sqlx::query(&insert_sql)
            .bind(migration.version)
            .bind(&migration.name)
            .bind(&migration.checksum)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to record migration: {}", e)))?;

        // 提交事务
        tx.commit()
            .await
            .map_err(|e| AppError::database(format!("Failed to commit migration: {}", e)))?;

        info!(
            version = migration.version,
            name = %migration.name,
            "Migration applied successfully"
        );

        Ok(())
    }

    /// 回滚单个迁移
    pub async fn rollback(&self, migration: &Migration) -> AppResult<()> {
        let table = self.full_table_name();

        let down_sql = migration.down_sql.as_ref().ok_or_else(|| {
            AppError::internal(format!(
                "Migration {} does not have down SQL",
                migration.version
            ))
        })?;

        // 开始事务
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        // 执行回滚 SQL
        sqlx::query(down_sql).execute(&mut *tx).await.map_err(|e| {
            AppError::database(format!(
                "Failed to rollback migration {}: {}",
                migration.version, e
            ))
        })?;

        // 删除迁移记录
        let delete_sql = format!("DELETE FROM {} WHERE version = $1", table);
        sqlx::query(&delete_sql)
            .bind(migration.version)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to delete migration record: {}", e)))?;

        // 提交事务
        tx.commit()
            .await
            .map_err(|e| AppError::database(format!("Failed to commit rollback: {}", e)))?;

        info!(
            version = migration.version,
            name = %migration.name,
            "Migration rolled back successfully"
        );

        Ok(())
    }

    /// 应用所有待处理的迁移
    pub async fn migrate(&self, migrations: &[Migration]) -> AppResult<MigrationResult> {
        self.init().await?;

        let applied = self.get_applied_migrations().await?;
        let applied_versions: std::collections::HashSet<i64> =
            applied.iter().map(|m| m.version).collect();

        let mut result = MigrationResult {
            applied: Vec::new(),
            skipped: Vec::new(),
            errors: Vec::new(),
        };

        // 按版本排序
        let mut sorted_migrations: Vec<_> = migrations.iter().collect();
        sorted_migrations.sort_by_key(|m| m.version);

        for migration in sorted_migrations {
            if applied_versions.contains(&migration.version) {
                // 验证校验和
                if let Some(record) = applied.iter().find(|r| r.version == migration.version) {
                    if record.checksum != migration.checksum {
                        result.errors.push(MigrationError {
                            version: migration.version,
                            name: migration.name.clone(),
                            error: "Checksum mismatch - migration has been modified".to_string(),
                        });
                        continue;
                    }
                }
                result.skipped.push(migration.version);
                continue;
            }

            match self.apply(migration).await {
                Ok(()) => {
                    result.applied.push(migration.version);
                }
                Err(e) => {
                    result.errors.push(MigrationError {
                        version: migration.version,
                        name: migration.name.clone(),
                        error: e.to_string(),
                    });
                    // 停止后续迁移
                    break;
                }
            }
        }

        Ok(result)
    }

    /// 回滚到指定版本
    pub async fn rollback_to(&self, target_version: i64, migrations: &[Migration]) -> AppResult<u32> {
        let applied = self.get_applied_migrations().await?;
        let mut rollback_count = 0;

        // 按版本降序排列
        let mut to_rollback: Vec<_> = applied
            .iter()
            .filter(|m| m.version > target_version)
            .collect();
        to_rollback.sort_by(|a, b| b.version.cmp(&a.version));

        for record in to_rollback {
            let migration = migrations
                .iter()
                .find(|m| m.version == record.version)
                .ok_or_else(|| {
                    AppError::internal(format!(
                        "Migration {} not found in provided migrations",
                        record.version
                    ))
                })?;

            self.rollback(migration).await?;
            rollback_count += 1;
        }

        Ok(rollback_count)
    }
}

/// 迁移结果
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// 已应用的迁移版本
    pub applied: Vec<i64>,
    /// 跳过的迁移版本
    pub skipped: Vec<i64>,
    /// 错误
    pub errors: Vec<MigrationError>,
}

impl MigrationResult {
    /// 是否成功
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// 应用的迁移数量
    pub fn applied_count(&self) -> usize {
        self.applied.len()
    }
}

/// 迁移错误
#[derive(Debug, Clone)]
pub struct MigrationError {
    /// 版本
    pub version: i64,
    /// 名称
    pub name: String,
    /// 错误信息
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_creation() {
        let migration = Migration::new(1, "create_users", "CREATE TABLE users (id SERIAL PRIMARY KEY)")
            .with_down("DROP TABLE users");

        assert_eq!(migration.version, 1);
        assert_eq!(migration.name, "create_users");
        assert!(migration.down_sql.is_some());
        assert!(!migration.checksum.is_empty());
    }

    #[test]
    fn test_checksum_consistency() {
        let sql = "CREATE TABLE test (id INT)";
        let m1 = Migration::new(1, "test", sql);
        let m2 = Migration::new(1, "test", sql);

        assert_eq!(m1.checksum, m2.checksum);
    }

    #[test]
    fn test_checksum_difference() {
        let m1 = Migration::new(1, "test", "CREATE TABLE test1 (id INT)");
        let m2 = Migration::new(1, "test", "CREATE TABLE test2 (id INT)");

        assert_ne!(m1.checksum, m2.checksum);
    }

    #[test]
    fn test_migration_result() {
        let result = MigrationResult {
            applied: vec![1, 2, 3],
            skipped: vec![],
            errors: vec![],
        };

        assert!(result.is_success());
        assert_eq!(result.applied_count(), 3);
    }
}
