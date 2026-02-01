//! Unit of Work 模式
//!
//! 提供跨多个 Repository 的事务协调能力，确保操作的原子性。

use async_trait::async_trait;
use errors::AppResult;

use crate::domain::repositories::{
    MaterialGroupRepository, MaterialRepository, MaterialTypeRepository,
};

/// Unit of Work trait
///
/// 协调多个 Repository 在同一事务中的操作。
///
/// # 使用示例
///
/// ```ignore
/// let uow = uow_factory.begin().await?;
///
/// // 所有操作在同一事务中
/// uow.materials().save(&material).await?;
/// uow.material_groups().save(&group).await?;
///
/// // 提交事务
/// uow.commit().await?;
/// ```
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// 获取物料 Repository
    fn materials(&self) -> &dyn MaterialRepository;

    /// 获取物料组 Repository
    fn material_groups(&self) -> &dyn MaterialGroupRepository;

    /// 获取物料类型 Repository
    fn material_types(&self) -> &dyn MaterialTypeRepository;

    /// 提交事务
    ///
    /// 成功时所有更改将持久化，失败时自动回滚。
    async fn commit(self: Box<Self>) -> AppResult<()>;

    /// 回滚事务
    ///
    /// 撤销所有未提交的更改。
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}

/// Unit of Work 工厂 trait
///
/// 用于创建新的 UnitOfWork 实例。
#[async_trait]
pub trait UnitOfWorkFactory: Send + Sync {
    /// 开始新的事务
    async fn begin(&self) -> AppResult<Box<dyn UnitOfWork>>;
}
