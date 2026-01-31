//! Repository trait 定义

use async_trait::async_trait;
use common::Pagination;
use errors::AppResult;

/// 基础 Repository trait
#[async_trait]
pub trait Repository<T, ID>: Send + Sync {
    /// 根据 ID 查找
    async fn find_by_id(&self, id: &ID) -> AppResult<Option<T>>;

    /// 保存实体
    async fn save(&self, entity: &T) -> AppResult<()>;

    /// 删除实体
    async fn delete(&self, id: &ID) -> AppResult<()>;

    /// 检查是否存在
    async fn exists(&self, id: &ID) -> AppResult<bool>;
}

/// 支持分页查询的 Repository
#[async_trait]
pub trait PageableRepository<T, ID>: Repository<T, ID> {
    /// 分页查询所有
    async fn find_all(&self, pagination: &Pagination) -> AppResult<Vec<T>>;

    /// 统计总数
    async fn count(&self) -> AppResult<u64>;
}
