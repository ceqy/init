//! 物料仓储接口

use async_trait::async_trait;
use common::types::{PagedResult, Pagination, TenantId};
use errors::AppResult;

use crate::domain::entities::{Material, MaterialFilter, MaterialSearchResult};
use crate::domain::value_objects::{AlternativeMaterial, MaterialId, MaterialNumber, UnitConversion};
use crate::domain::views::{
    AccountingData, PlantData, PurchaseData, QualityData, SalesData, StorageData,
};

/// 物料仓储接口
#[async_trait]
pub trait MaterialRepository: Send + Sync {
    // ========== CRUD ==========

    /// 根据 ID 查找物料
    async fn find_by_id(
        &self,
        id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<Material>>;

    /// 根据物料编号查找物料
    async fn find_by_number(
        &self,
        number: &MaterialNumber,
        tenant_id: &TenantId,
    ) -> AppResult<Option<Material>>;

    /// 保存物料（新建）
    async fn save(&self, material: &Material) -> AppResult<()>;

    /// 更新物料
    async fn update(&self, material: &Material) -> AppResult<()>;

    /// 删除物料
    async fn delete(&self, id: &MaterialId, tenant_id: &TenantId) -> AppResult<()>;

    // ========== 查询 ==========

    /// 列表查询
    async fn list(
        &self,
        tenant_id: &TenantId,
        filter: MaterialFilter,
        pagination: Pagination,
    ) -> AppResult<PagedResult<Material>>;

    /// 搜索物料
    async fn search(
        &self,
        tenant_id: &TenantId,
        query: &str,
        pagination: Pagination,
    ) -> AppResult<Vec<MaterialSearchResult>>;

    /// 检查物料编号是否存在
    async fn exists_by_number(
        &self,
        number: &MaterialNumber,
        tenant_id: &TenantId,
    ) -> AppResult<bool>;

    // ========== 替代物料 ==========

    /// 查找替代物料
    async fn find_alternatives(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<AlternativeMaterial>>;

    /// 保存替代物料关系
    async fn save_alternative(
        &self,
        material_id: &MaterialId,
        alternative: &AlternativeMaterial,
    ) -> AppResult<()>;

    /// 移除替代物料关系
    async fn remove_alternative(
        &self,
        material_id: &MaterialId,
        alternative_id: &MaterialId,
    ) -> AppResult<()>;

    // ========== 视图数据 ==========

    /// 保存工厂数据
    async fn save_plant_data(
        &self,
        material_id: &MaterialId,
        plant_data: &PlantData,
    ) -> AppResult<()>;

    /// 获取工厂数据
    async fn get_plant_data(
        &self,
        material_id: &MaterialId,
        plant: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PlantData>>;

    /// 保存销售数据
    async fn save_sales_data(
        &self,
        material_id: &MaterialId,
        sales_data: &SalesData,
    ) -> AppResult<()>;

    /// 获取销售数据
    async fn get_sales_data(
        &self,
        material_id: &MaterialId,
        sales_org: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<SalesData>>;

    /// 保存采购数据
    async fn save_purchase_data(
        &self,
        material_id: &MaterialId,
        purchase_data: &PurchaseData,
    ) -> AppResult<()>;

    /// 获取采购数据
    async fn get_purchase_data(
        &self,
        material_id: &MaterialId,
        purchase_org: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PurchaseData>>;

    /// 保存仓储数据
    async fn save_storage_data(
        &self,
        material_id: &MaterialId,
        storage_data: &StorageData,
    ) -> AppResult<()>;

    /// 获取仓储数据
    async fn get_storage_data(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<StorageData>>;

    /// 保存会计数据
    async fn save_accounting_data(
        &self,
        material_id: &MaterialId,
        accounting_data: &AccountingData,
    ) -> AppResult<()>;

    /// 获取会计数据
    async fn get_accounting_data(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<AccountingData>>;

    /// 保存质量数据
    async fn save_quality_data(
        &self,
        material_id: &MaterialId,
        quality_data: &QualityData,
    ) -> AppResult<()>;

    /// 获取质量数据
    async fn get_quality_data(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<QualityData>>;

    // ========== 单位换算 ==========

    /// 查找单位换算
    async fn find_unit_conversions(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<UnitConversion>>;

    /// 保存单位换算
    async fn save_unit_conversion(
        &self,
        material_id: &MaterialId,
        conversion: &UnitConversion,
    ) -> AppResult<()>;

    /// 删除单位换算
    async fn delete_unit_conversion(
        &self,
        material_id: &MaterialId,
        from_unit: &str,
        to_unit: &str,
    ) -> AppResult<()>;
}
