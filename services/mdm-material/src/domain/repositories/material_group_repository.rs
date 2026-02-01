//! 物料组仓储接口

use async_trait::async_trait;
use common::types::{PagedResult, Pagination, TenantId};

use crate::domain::entities::MaterialGroup;
use crate::domain::value_objects::MaterialGroupId;
use crate::error::ServiceResult;

/// 物料组仓储接口
#[async_trait]
pub trait MaterialGroupRepository: Send + Sync {
    /// 根据 ID 查找物料组
    async fn find_by_id(
        &self,
        id: &MaterialGroupId,
        tenant_id: &TenantId,
    ) -> ServiceResult<Option<MaterialGroup>>;

    /// 根据编码查找物料组
    async fn find_by_code(
        &self,
        code: &str,
        tenant_id: &TenantId,
    ) -> ServiceResult<Option<MaterialGroup>>;

    /// 保存物料组（新建）
    async fn save(&self, group: &MaterialGroup) -> ServiceResult<()>;

    /// 更新物料组
    async fn update(&self, group: &MaterialGroup) -> ServiceResult<()>;

    /// 删除物料组
    async fn delete(&self, id: &MaterialGroupId, tenant_id: &TenantId) -> ServiceResult<()>;

    /// 列表查询
    async fn list(
        &self,
        tenant_id: &TenantId,
        parent_id: Option<&MaterialGroupId>,
        pagination: Pagination,
    ) -> ServiceResult<PagedResult<MaterialGroup>>;

    /// 查找子级物料组
    async fn find_children(
        &self,
        parent_id: &MaterialGroupId,
        tenant_id: &TenantId,
    ) -> ServiceResult<Vec<MaterialGroup>>;

    /// 检查编码是否存在
    async fn exists_by_code(&self, code: &str, tenant_id: &TenantId) -> ServiceResult<bool>;
}
