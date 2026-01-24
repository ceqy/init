//! Unit of Work trait 定义

use async_trait::async_trait;
use cuba_errors::AppResult;

/// Unit of Work trait
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// 开始事务
    async fn begin(&mut self) -> AppResult<()>;

    /// 提交事务
    async fn commit(&mut self) -> AppResult<()>;

    /// 回滚事务
    async fn rollback(&mut self) -> AppResult<()>;
}
