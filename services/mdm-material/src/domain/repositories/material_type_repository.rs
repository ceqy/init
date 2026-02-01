//! 物料类型仓储接口

use async_trait::async_trait;
use common::types::{PagedResult, Pagination, TenantId};
use errors::AppResult;

use crate::domain::entities::MaterialType;
use crate::domain::value_objects::MaterialTypeId;

/// 物料类型仓储接口
#[async_trait]
pub trait MaterialTypeRepository: Send + Sync {
    /// 根据 ID 查找物料类型
    async fn find_by_id(
        &self,
        id: &MaterialTypeId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<MaterialType>>;

    /// 根据编码查找物料类型
    async fn find_by_code(
        &self,
        code: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<MaterialType>>;

    /// 保存物料类型（新建）
    async fn save(&self, material_type: &MaterialType) -> AppResult<()>;

    /// 更新物料类型
    async fn update(&self, material_type: &MaterialType) -> AppResult<()>;

    /// 列表查询
    async fn list(
        &self,
        tenant_id: &TenantId,
        pagination: Pagination,
    ) -> AppResult<PagedResult<MaterialType>>;

    /// 检查编码是否存在
    async fn exists_by_code(&self, code: &str, tenant_id: &TenantId) -> AppResult<bool>;
}
