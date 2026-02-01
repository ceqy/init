//! 物料仓储接口

use async_trait::async_trait;
use common::types::{PagedResult, Pagination, TenantId};

use crate::domain::entities::{Material, MaterialFilter, MaterialSearchResult};
use crate::domain::value_objects::{AlternativeMaterial, MaterialId, MaterialNumber};
use crate::error::ServiceResult;

/// 物料仓储接口
#[async_trait]
pub trait MaterialRepository: Send + Sync {
    // ========== CRUD ==========

    /// 根据 ID 查找物料
    async fn find_by_id(
        &self,
        id: &MaterialId,
        tenant_id: &TenantId,
    ) -> ServiceResult<Option<Material>>;

    /// 根据物料编号查找物料
    async fn find_by_number(
        &self,
        number: &MaterialNumber,
        tenant_id: &TenantId,
    ) -> ServiceResult<Option<Material>>;

    /// 保存物料（新建）
    async fn save(&self, material: &Material) -> ServiceResult<()>;

    /// 更新物料
    async fn update(&self, material: &Material) -> ServiceResult<()>;

    /// 删除物料
    async fn delete(&self, id: &MaterialId, tenant_id: &TenantId) -> ServiceResult<()>;

    // ========== 查询 ==========

    /// 列表查询
    async fn list(
        &self,
        tenant_id: &TenantId,
        filter: MaterialFilter,
        pagination: Pagination,
    ) -> ServiceResult<PagedResult<Material>>;

    /// 搜索物料
    async fn search(
        &self,
        tenant_id: &TenantId,
        query: &str,
        pagination: Pagination,
    ) -> ServiceResult<Vec<MaterialSearchResult>>;

    /// 检查物料编号是否存在
    async fn exists_by_number(
        &self,
        number: &MaterialNumber,
        tenant_id: &TenantId,
    ) -> ServiceResult<bool>;

    // ========== 替代物料 ==========

    /// 查找替代物料
    async fn find_alternatives(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
    ) -> ServiceResult<Vec<AlternativeMaterial>>;

    /// 保存替代物料关系
    async fn save_alternative(
        &self,
        material_id: &MaterialId,
        alternative: &AlternativeMaterial,
    ) -> ServiceResult<()>;

    /// 移除替代物料关系
    async fn remove_alternative(
        &self,
        material_id: &MaterialId,
        alternative_id: &MaterialId,
    ) -> ServiceResult<()>;
}
