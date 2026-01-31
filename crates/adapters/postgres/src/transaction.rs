//! PostgreSQL 事务管理模块
//!
//! 提供事务管理器和分布式事务支持

use errors::{AppError, AppResult};
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// 事务隔离级别
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum IsolationLevel {
    /// 读未提交
    ReadUncommitted,
    /// 读已提交（PostgreSQL 默认）
    #[default]
    ReadCommitted,
    /// 可重复读
    RepeatableRead,
    /// 可串行化
    Serializable,
}

impl IsolationLevel {
    /// 转换为 SQL 字符串
    pub fn as_sql(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }
}

/// 事务访问模式
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AccessMode {
    /// 读写
    #[default]
    ReadWrite,
    /// 只读
    ReadOnly,
}

impl AccessMode {
    /// 转换为 SQL 字符串
    pub fn as_sql(&self) -> &'static str {
        match self {
            AccessMode::ReadWrite => "READ WRITE",
            AccessMode::ReadOnly => "READ ONLY",
        }
    }
}

/// 事务选项
#[derive(Debug, Clone, Default)]
pub struct TransactionOptions {
    /// 隔离级别
    pub isolation_level: IsolationLevel,
    /// 访问模式
    pub access_mode: AccessMode,
    /// 是否可延迟（仅对 SERIALIZABLE READ ONLY 有效）
    pub deferrable: bool,
}

impl TransactionOptions {
    /// 创建新的事务选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置隔离级别
    pub fn with_isolation_level(mut self, level: IsolationLevel) -> Self {
        self.isolation_level = level;
        self
    }

    /// 设置为只读
    pub fn read_only(mut self) -> Self {
        self.access_mode = AccessMode::ReadOnly;
        self
    }

    /// 设置为可串行化
    pub fn serializable(mut self) -> Self {
        self.isolation_level = IsolationLevel::Serializable;
        self
    }

    /// 设置为可延迟
    pub fn deferrable(mut self) -> Self {
        self.deferrable = true;
        self
    }

    /// 生成 SET TRANSACTION 语句
    pub fn to_sql(&self) -> String {
        let mut parts = vec![
            format!("ISOLATION LEVEL {}", self.isolation_level.as_sql()),
            self.access_mode.as_sql().to_string(),
        ];

        if self.deferrable
            && self.isolation_level == IsolationLevel::Serializable
            && self.access_mode == AccessMode::ReadOnly
        {
            parts.push("DEFERRABLE".to_string());
        }

        format!("SET TRANSACTION {}", parts.join(", "))
    }
}

/// 事务管理器
#[derive(Clone)]
pub struct TransactionManager {
    pool: PgPool,
}

impl TransactionManager {
    /// 创建新的事务管理器
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 开始事务
    pub async fn begin(&self) -> AppResult<Transaction<'static, Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))
    }

    /// 开始带选项的事务
    pub async fn begin_with_options(
        &self,
        options: &TransactionOptions,
    ) -> AppResult<Transaction<'static, Postgres>> {
        let mut tx = self.begin().await?;

        sqlx::query(&options.to_sql())
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to set transaction options: {}", e)))?;

        Ok(tx)
    }

    /// 开始只读事务
    pub async fn begin_readonly(&self) -> AppResult<Transaction<'static, Postgres>> {
        let options = TransactionOptions::new().read_only();
        self.begin_with_options(&options).await
    }

    /// 开始可串行化事务
    pub async fn begin_serializable(&self) -> AppResult<Transaction<'static, Postgres>> {
        let options = TransactionOptions::new().serializable();
        self.begin_with_options(&options).await
    }

    /// 提交事务
    pub async fn commit(tx: Transaction<'static, Postgres>) -> AppResult<()> {
        tx.commit()
            .await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))
    }

    /// 回滚事务
    pub async fn rollback(tx: Transaction<'static, Postgres>) -> AppResult<()> {
        tx.rollback()
            .await
            .map_err(|e| AppError::database(format!("Failed to rollback transaction: {}", e)))
    }
}

/// Savepoint 管理
pub struct Savepoint<'a> {
    tx: &'a mut Transaction<'static, Postgres>,
    name: String,
    released: bool,
}

impl<'a> Savepoint<'a> {
    /// 创建新的 savepoint
    pub async fn new(
        tx: &'a mut Transaction<'static, Postgres>,
        name: impl Into<String>,
    ) -> AppResult<Savepoint<'a>> {
        let name = name.into();
        sqlx::query(&format!("SAVEPOINT {}", name))
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to create savepoint: {}", e)))?;

        debug!(savepoint = %name, "Savepoint created");

        Ok(Savepoint {
            tx,
            name,
            released: false,
        })
    }

    /// 释放 savepoint
    pub async fn release(mut self) -> AppResult<()> {
        sqlx::query(&format!("RELEASE SAVEPOINT {}", self.name))
            .execute(&mut **self.tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to release savepoint: {}", e)))?;

        self.released = true;
        debug!(savepoint = %self.name, "Savepoint released");
        Ok(())
    }

    /// 回滚到 savepoint
    pub async fn rollback(mut self) -> AppResult<()> {
        sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", self.name))
            .execute(&mut **self.tx)
            .await
            .map_err(|e| AppError::database(format!("Failed to rollback to savepoint: {}", e)))?;

        self.released = true;
        debug!(savepoint = %self.name, "Rolled back to savepoint");
        Ok(())
    }
}

/// 两阶段提交（2PC）协调器
pub struct TwoPhaseCommit {
    pool: PgPool,
}

impl TwoPhaseCommit {
    /// 创建新的 2PC 协调器
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 生成事务 ID
    pub fn generate_xid() -> String {
        format!("tx_{}", Uuid::new_v4().to_string().replace('-', ""))
    }

    /// 准备事务（第一阶段）
    pub async fn prepare(&self, xid: &str) -> AppResult<()> {
        sqlx::query(&format!("PREPARE TRANSACTION '{}'", xid))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to prepare transaction: {}", e)))?;

        info!(xid = %xid, "Transaction prepared");
        Ok(())
    }

    /// 提交已准备的事务（第二阶段）
    pub async fn commit_prepared(&self, xid: &str) -> AppResult<()> {
        sqlx::query(&format!("COMMIT PREPARED '{}'", xid))
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::database(format!("Failed to commit prepared transaction: {}", e))
            })?;

        info!(xid = %xid, "Prepared transaction committed");
        Ok(())
    }

    /// 回滚已准备的事务
    pub async fn rollback_prepared(&self, xid: &str) -> AppResult<()> {
        sqlx::query(&format!("ROLLBACK PREPARED '{}'", xid))
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::database(format!("Failed to rollback prepared transaction: {}", e))
            })?;

        warn!(xid = %xid, "Prepared transaction rolled back");
        Ok(())
    }

    /// 获取所有已准备的事务
    pub async fn list_prepared(&self) -> AppResult<Vec<PreparedTransaction>> {
        let rows = sqlx::query_as::<_, PreparedTransactionRow>(
            r#"
            SELECT gid, prepared, owner, database
            FROM pg_prepared_xacts
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list prepared transactions: {}", e)))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// 清理过期的已准备事务
    pub async fn cleanup_stale(&self, max_age: chrono::Duration) -> AppResult<u32> {
        let cutoff = chrono::Utc::now() - max_age;
        let prepared = self.list_prepared().await?;
        let mut cleaned = 0;

        for tx in prepared {
            if tx.prepared < cutoff {
                match self.rollback_prepared(&tx.gid).await {
                    Ok(()) => {
                        cleaned += 1;
                        warn!(
                            xid = %tx.gid,
                            prepared = %tx.prepared,
                            "Cleaned up stale prepared transaction"
                        );
                    }
                    Err(e) => {
                        error!(
                            xid = %tx.gid,
                            error = %e,
                            "Failed to cleanup stale prepared transaction"
                        );
                    }
                }
            }
        }

        Ok(cleaned)
    }
}

/// 已准备的事务信息
#[derive(Debug, Clone)]
pub struct PreparedTransaction {
    /// 事务 ID
    pub gid: String,
    /// 准备时间
    pub prepared: chrono::DateTime<chrono::Utc>,
    /// 所有者
    pub owner: String,
    /// 数据库
    pub database: String,
}

#[derive(sqlx::FromRow)]
struct PreparedTransactionRow {
    gid: String,
    prepared: chrono::DateTime<chrono::Utc>,
    owner: String,
    database: String,
}

impl From<PreparedTransactionRow> for PreparedTransaction {
    fn from(row: PreparedTransactionRow) -> Self {
        Self {
            gid: row.gid,
            prepared: row.prepared,
            owner: row.owner,
            database: row.database,
        }
    }
}

/// 事务上下文（用于传递事务信息）
#[derive(Debug, Clone)]
pub struct TransactionContext {
    /// 事务 ID
    pub id: Uuid,
    /// 开始时间
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 隔离级别
    pub isolation_level: IsolationLevel,
    /// 是否只读
    pub read_only: bool,
}

impl TransactionContext {
    /// 创建新的事务上下文
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            started_at: chrono::Utc::now(),
            isolation_level: IsolationLevel::default(),
            read_only: false,
        }
    }

    /// 设置隔离级别
    pub fn with_isolation_level(mut self, level: IsolationLevel) -> Self {
        self.isolation_level = level;
        self
    }

    /// 设置为只读
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }
}

impl Default for TransactionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_level() {
        assert_eq!(IsolationLevel::ReadUncommitted.as_sql(), "READ UNCOMMITTED");
        assert_eq!(IsolationLevel::ReadCommitted.as_sql(), "READ COMMITTED");
        assert_eq!(IsolationLevel::RepeatableRead.as_sql(), "REPEATABLE READ");
        assert_eq!(IsolationLevel::Serializable.as_sql(), "SERIALIZABLE");
    }

    #[test]
    fn test_access_mode() {
        assert_eq!(AccessMode::ReadWrite.as_sql(), "READ WRITE");
        assert_eq!(AccessMode::ReadOnly.as_sql(), "READ ONLY");
    }

    #[test]
    fn test_transaction_options_sql() {
        let options = TransactionOptions::new()
            .with_isolation_level(IsolationLevel::Serializable)
            .read_only();

        let sql = options.to_sql();
        assert!(sql.contains("SERIALIZABLE"));
        assert!(sql.contains("READ ONLY"));
    }

    #[test]
    fn test_deferrable_option() {
        let options = TransactionOptions::new()
            .serializable()
            .read_only()
            .deferrable();

        let sql = options.to_sql();
        assert!(sql.contains("DEFERRABLE"));
    }

    #[test]
    fn test_transaction_context() {
        let ctx = TransactionContext::new()
            .with_isolation_level(IsolationLevel::RepeatableRead)
            .read_only();

        assert_eq!(ctx.isolation_level, IsolationLevel::RepeatableRead);
        assert!(ctx.read_only);
    }

    #[test]
    fn test_generate_xid() {
        let xid1 = TwoPhaseCommit::generate_xid();
        let xid2 = TwoPhaseCommit::generate_xid();

        assert!(xid1.starts_with("tx_"));
        assert!(xid2.starts_with("tx_"));
        assert_ne!(xid1, xid2);
    }
}
