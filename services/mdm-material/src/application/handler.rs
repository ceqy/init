//! Business logic handler

use std::sync::Arc;

use common::types::PagedResult;
use domain_core::{AggregateRoot, Entity};
use errors::{AppError, AppResult};
use tracing::{info, warn};

use crate::domain::entities::{Material, MaterialGroup, MaterialSearchResult, MaterialType};
use crate::domain::repositories::{
    MaterialGroupRepository, MaterialRepository, MaterialTypeRepository,
};
use crate::domain::value_objects::{MaterialGroupId, MaterialId, MaterialNumber, MaterialTypeId};
use crate::domain::views::{
    AccountingData, PlantData, PurchaseData, QualityData, SalesData, StorageData,
};

use super::commands::*;
use super::queries::*;

pub struct ServiceHandler {
    material_repo: Arc<dyn MaterialRepository>,
    group_repo: Arc<dyn MaterialGroupRepository>,
    type_repo: Arc<dyn MaterialTypeRepository>,
}

impl ServiceHandler {
    pub fn new(
        material_repo: Arc<dyn MaterialRepository>,
        group_repo: Arc<dyn MaterialGroupRepository>,
        type_repo: Arc<dyn MaterialTypeRepository>,
    ) -> Self {
        Self {
            material_repo,
            group_repo,
            type_repo,
        }
    }

    // ========== 物料基础 CRUD ==========

    /// 创建物料
    pub async fn create_material(&self, cmd: CreateMaterialCommand) -> AppResult<MaterialId> {
        info!(
            "Creating material: {} for tenant: {}",
            cmd.material_number, cmd.tenant_id.0
        );

        // 1. 验证命令
        cmd.validate()?;

        // 2. 检查物料编号是否已存在
        let material_number = MaterialNumber::new(cmd.material_number.clone())
            .map_err(|e| AppError::validation(e.to_string()))?;
        let exists = self
            .material_repo
            .exists_by_number(&material_number, &cmd.tenant_id)
            .await?;

        if exists {
            return Err(AppError::conflict(format!(
                "物料编号 {} 已存在",
                cmd.material_number
            )));
        }

        // 3. 创建物料实体
        let mut material = Material::new(
            cmd.tenant_id.clone(),
            material_number,
            cmd.description.clone(),
            cmd.material_type_id.clone(),
            cmd.material_type_id.to_string(),
            cmd.base_unit.clone(),
        );

        // 设置审计信息
        {
            let audit = material.audit_info_mut();
            audit.created_by = Some(cmd.user_id);
            audit.created_at = chrono::Utc::now();
        }

        let material_id = material.id().clone();

        // 设置可选字段
        if let Some(localized) = cmd.localized_description {
            material = material.with_localized_description(localized);
        }
        if let Some(group_id) = cmd.material_group_id {
            material = material.with_material_group(group_id, "");
        }

        // 设置尺寸和重量
        if let (Some(gross_weight), Some(net_weight)) = (cmd.gross_weight, cmd.net_weight) {
            if let Some(unit) = cmd.weight_unit {
                material = material.with_weight(gross_weight, net_weight, unit);
            }
        }
        if let Some(volume) = cmd.volume {
            if let Some(unit) = cmd.volume_unit {
                material = material.with_volume(volume, unit);
            }
        }
        if let (Some(length), Some(width), Some(height)) = (cmd.length, cmd.width, cmd.height) {
            if let Some(unit) = cmd.dimension_unit {
                material = material.with_dimensions(length, width, height, unit);
            }
        }

        // 设置扩展属性
        if let Some(attrs) = cmd.custom_attributes {
            // 将 serde_json::Value 转换为 HashMap<String, String>
            if let Ok(map) = serde_json::from_value::<std::collections::HashMap<String, String>>(attrs) {
                material = material.with_custom_attributes(map);
            }
        }

        // 4. 保存到数据库
        self.material_repo.save(&material).await?;

        info!("Material created successfully: {}", material_id.0);
        Ok(material_id)
    }

    /// 获取物料
    pub async fn get_material(&self, query: GetMaterialQuery) -> AppResult<Material> {
        info!(
            "Getting material: {} for tenant: {}",
            query.material_id.0, query.tenant_id.0
        );

        let material = self
            .material_repo
            .find_by_id(&query.material_id, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        Ok(material)
    }

    /// 按编号获取物料
    pub async fn get_material_by_number(
        &self,
        query: GetMaterialByNumberQuery,
    ) -> AppResult<Material> {
        info!(
            "Getting material by number: {} for tenant: {}",
            query.material_number.as_str(),
            query.tenant_id.0
        );

        let material = self
            .material_repo
            .find_by_number(&query.material_number, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        Ok(material)
    }

    /// 更新物料
    pub async fn update_material(&self, cmd: UpdateMaterialCommand) -> AppResult<()> {
        info!(
            "Updating material: {} for tenant: {}",
            cmd.material_id.0, cmd.tenant_id.0
        );

        // 1. 验证命令
        cmd.validate()?;

        // 2. 获取现有物料
        let mut material = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        // 3. 更新字段
        // 注意：由于 Material 实体没有提供所有字段的 setter 方法，
        // 这里只更新有 builder 方法的字段
        if let Some(localized) = cmd.localized_description {
            material = material.with_localized_description(localized);
        }
        if let Some(group_id) = cmd.material_group_id {
            // with_material_group 需要 group_code，这里使用空字符串
            material = material.with_material_group(group_id, "");
        }

        // 更新尺寸和重量
        if let (Some(gross_weight), Some(net_weight)) = (cmd.gross_weight, cmd.net_weight) {
            if let Some(unit) = cmd.weight_unit {
                material = material.with_weight(gross_weight, net_weight, unit);
            }
        }
        if let Some(volume) = cmd.volume {
            if let Some(unit) = cmd.volume_unit {
                material = material.with_volume(volume, unit);
            }
        }
        if let (Some(length), Some(width), Some(height)) = (cmd.length, cmd.width, cmd.height) {
            if let Some(unit) = cmd.dimension_unit {
                material = material.with_dimensions(length, width, height, unit);
            }
        }

        // 更新扩展属性
        if let Some(attrs) = cmd.custom_attributes {
            // 将 serde_json::Value 转换为 HashMap<String, String>
            if let Ok(map) = serde_json::from_value::<std::collections::HashMap<String, String>>(attrs) {
                material = material.with_custom_attributes(map);
            }
        }

        // 更新审计信息
        {
            let audit = material.audit_info_mut();
            audit.updated_by = Some(cmd.user_id);
            audit.updated_at = chrono::Utc::now();
        }

        // 4. 保存更新
        self.material_repo.update(&material).await?;

        info!("Material updated successfully: {}", cmd.material_id.0);
        Ok(())
    }

    /// 删除物料
    pub async fn delete_material(&self, cmd: DeleteMaterialCommand) -> AppResult<()> {
        info!(
            "Deleting material: {} for tenant: {}",
            cmd.material_id.0, cmd.tenant_id.0
        );

        // 1. 检查物料是否存在
        let material = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?;

        if material.is_none() {
            warn!("Material not found: {}", cmd.material_id.0);
            return Err(AppError::not_found("物料不存在"));
        }

        // 2. 执行删除
        self.material_repo
            .delete(&cmd.material_id, &cmd.tenant_id)
            .await?;

        info!("Material deleted successfully: {}", cmd.material_id.0);
        Ok(())
    }

    /// 列表查询
    pub async fn list_materials(
        &self,
        query: ListMaterialsQuery,
    ) -> AppResult<PagedResult<Material>> {
        info!(
            "Listing materials for tenant: {}, page: {}, size: {}",
            query.tenant_id.0, query.pagination.page, query.pagination.page_size
        );

        let result = self
            .material_repo
            .list(&query.tenant_id, query.filter, query.pagination)
            .await?;

        info!("Found {} materials", result.total);
        Ok(result)
    }

    /// 搜索物料
    pub async fn search_materials(
        &self,
        query: SearchMaterialsQuery,
    ) -> AppResult<Vec<MaterialSearchResult>> {
        info!(
            "Searching materials: '{}' for tenant: {}",
            query.query, query.tenant_id.0
        );

        let results = self
            .material_repo
            .search(&query.tenant_id, &query.query, query.pagination)
            .await?;

        info!("Found {} search results", results.len());
        Ok(results)
    }

    // ========== 视图扩展 ==========

    /// 扩展到工厂
    pub async fn extend_to_plant(&self, cmd: ExtendToPlantCommand) -> AppResult<()> {
        info!(
            "Extending material {} to plant: {}",
            cmd.material_id.0, cmd.plant_data.plant()
        );

        // 1. 检查物料是否存在
        let _material = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        // 2. 保存工厂数据
        self.material_repo
            .save_plant_data(&cmd.material_id, &cmd.plant_data)
            .await?;

        info!("Plant data saved successfully");
        Ok(())
    }

    /// 更新工厂数据
    pub async fn update_plant_data(&self, cmd: UpdatePlantDataCommand) -> AppResult<()> {
        info!(
            "Updating plant data for material {} at plant: {}",
            cmd.material_id.0, cmd.plant
        );

        // 1. 检查数据是否存在
        let existing = self
            .material_repo
            .get_plant_data(&cmd.material_id, &cmd.plant, &cmd.tenant_id)
            .await?;

        if existing.is_none() {
            return Err(AppError::not_found("工厂数据不存在"));
        }

        // 2. 更新数据
        self.material_repo
            .save_plant_data(&cmd.material_id, &cmd.plant_data)
            .await?;

        info!("Plant data updated successfully");
        Ok(())
    }

    /// 扩展到销售
    pub async fn extend_to_sales(&self, cmd: ExtendToSalesCommand) -> AppResult<()> {
        info!(
            "Extending material {} to sales org: {}",
            cmd.material_id.0, cmd.sales_data.sales_org()
        );

        // 检查物料是否存在
        let _ = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        // 保存销售数据
        self.material_repo
            .save_sales_data(&cmd.material_id, &cmd.sales_data)
            .await?;

        info!("Sales data saved successfully");
        Ok(())
    }

    /// 更新销售数据
    pub async fn update_sales_data(&self, cmd: UpdateSalesDataCommand) -> AppResult<()> {
        info!(
            "Updating sales data for material {} at sales org: {}",
            cmd.material_id.0, cmd.sales_org
        );

        // 检查数据是否存在
        let existing = self
            .material_repo
            .get_sales_data(&cmd.material_id, &cmd.sales_org, &cmd.tenant_id)
            .await?;

        if existing.is_none() {
            return Err(AppError::not_found("销售数据不存在"));
        }

        // 更新数据
        self.material_repo
            .save_sales_data(&cmd.material_id, &cmd.sales_data)
            .await?;

        info!("Sales data updated successfully");
        Ok(())
    }

    /// 扩展到采购
    pub async fn extend_to_purchase(&self, cmd: ExtendToPurchaseCommand) -> AppResult<()> {
        info!(
            "Extending material {} to purchase org: {}",
            cmd.material_id.0, cmd.purchase_data.purchase_org()
        );

        // 检查物料是否存在
        let _ = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        // 保存采购数据
        self.material_repo
            .save_purchase_data(&cmd.material_id, &cmd.purchase_data)
            .await?;

        info!("Purchase data saved successfully");
        Ok(())
    }

    /// 更新采购数据
    pub async fn update_purchase_data(&self, cmd: UpdatePurchaseDataCommand) -> AppResult<()> {
        info!(
            "Updating purchase data for material {} at purchase org: {}",
            cmd.material_id.0, cmd.purchase_org
        );

        // 检查数据是否存在
        let existing = self
            .material_repo
            .get_purchase_data(&cmd.material_id, &cmd.purchase_org, &cmd.tenant_id)
            .await?;

        if existing.is_none() {
            return Err(AppError::not_found("采购数据不存在"));
        }

        // 更新数据
        self.material_repo
            .save_purchase_data(&cmd.material_id, &cmd.purchase_data)
            .await?;

        info!("Purchase data updated successfully");
        Ok(())
    }

    /// 扩展到仓储
    pub async fn extend_to_storage(&self, cmd: ExtendToStorageCommand) -> AppResult<()> {
        info!("Extending material {} to storage", cmd.material_id.0);

        // 检查物料是否存在
        let _ = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        // 保存仓储数据
        self.material_repo
            .save_storage_data(&cmd.material_id, &cmd.storage_data)
            .await?;

        info!("Storage data saved successfully");
        Ok(())
    }

    /// 更新仓储数据
    pub async fn update_storage_data(&self, cmd: UpdateStorageDataCommand) -> AppResult<()> {
        info!("Updating storage data for material {}", cmd.material_id.0);

        // 检查数据是否存在
        let existing = self
            .material_repo
            .get_storage_data(&cmd.material_id, &cmd.tenant_id)
            .await?;

        if existing.is_none() {
            return Err(AppError::not_found("仓储数据不存在"));
        }

        // 更新数据
        self.material_repo
            .save_storage_data(&cmd.material_id, &cmd.storage_data)
            .await?;

        info!("Storage data updated successfully");
        Ok(())
    }

    /// 扩展到会计
    pub async fn extend_to_accounting(&self, cmd: ExtendToAccountingCommand) -> AppResult<()> {
        info!("Extending material {} to accounting", cmd.material_id.0);

        // 检查物料是否存在
        let _ = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        // 保存会计数据
        self.material_repo
            .save_accounting_data(&cmd.material_id, &cmd.accounting_data)
            .await?;

        info!("Accounting data saved successfully");
        Ok(())
    }

    /// 更新会计数据
    pub async fn update_accounting_data(
        &self,
        cmd: UpdateAccountingDataCommand,
    ) -> AppResult<()> {
        info!("Updating accounting data for material {}", cmd.material_id.0);

        // 检查数据是否存在
        let existing = self
            .material_repo
            .get_accounting_data(&cmd.material_id, &cmd.tenant_id)
            .await?;

        if existing.is_none() {
            return Err(AppError::not_found("会计数据不存在"));
        }

        // 更新数据
        self.material_repo
            .save_accounting_data(&cmd.material_id, &cmd.accounting_data)
            .await?;

        info!("Accounting data updated successfully");
        Ok(())
    }

    /// 扩展到质量
    pub async fn extend_to_quality(&self, cmd: ExtendToQualityCommand) -> AppResult<()> {
        info!("Extending material {} to quality", cmd.material_id.0);

        // 检查物料是否存在
        let _ = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        // 保存质量数据
        self.material_repo
            .save_quality_data(&cmd.material_id, &cmd.quality_data)
            .await?;

        info!("Quality data saved successfully");
        Ok(())
    }

    /// 更新质量数据
    pub async fn update_quality_data(&self, cmd: UpdateQualityDataCommand) -> AppResult<()> {
        info!("Updating quality data for material {}", cmd.material_id.0);

        // 检查数据是否存在
        let existing = self
            .material_repo
            .get_quality_data(&cmd.material_id, &cmd.tenant_id)
            .await?;

        if existing.is_none() {
            return Err(AppError::not_found("质量数据不存在"));
        }

        // 更新数据
        self.material_repo
            .save_quality_data(&cmd.material_id, &cmd.quality_data)
            .await?;

        info!("Quality data updated successfully");
        Ok(())
    }

    // ========== 状态管理 ==========

    /// 激活物料
    pub async fn activate_material(&self, cmd: ActivateMaterialCommand) -> AppResult<()> {
        info!("Activating material: {}", cmd.material_id.0);

        // 获取物料
        let mut material = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        // 激活
        material.activate();

        // 更新审计信息
        {
            let audit = material.audit_info_mut();
            audit.updated_by = Some(cmd.user_id);
            audit.updated_at = chrono::Utc::now();
        }

        // 保存
        self.material_repo.update(&material).await?;

        info!("Material activated successfully");
        Ok(())
    }

    /// 停用物料
    pub async fn deactivate_material(&self, cmd: DeactivateMaterialCommand) -> AppResult<()> {
        info!("Deactivating material: {}", cmd.material_id.0);

        // 获取物料
        let mut material = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        // 停用
        material.deactivate();

        // 更新审计信息
        {
            let audit = material.audit_info_mut();
            audit.updated_by = Some(cmd.user_id);
            audit.updated_at = chrono::Utc::now();
        }

        // 保存
        self.material_repo.update(&material).await?;

        info!("Material deactivated successfully");
        Ok(())
    }

    /// 冻结物料
    pub async fn block_material(&self, cmd: BlockMaterialCommand) -> AppResult<()> {
        info!("Blocking material: {}", cmd.material_id.0);

        // 获取物料
        let mut material = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        // 冻结
        material.block();

        // 更新审计信息
        {
            let audit = material.audit_info_mut();
            audit.updated_by = Some(cmd.user_id);
            audit.updated_at = chrono::Utc::now();
        }

        // 保存
        self.material_repo.update(&material).await?;

        info!("Material blocked successfully");
        Ok(())
    }

    /// 标记删除
    pub async fn mark_for_deletion(&self, cmd: MarkForDeletionCommand) -> AppResult<()> {
        info!("Marking material for deletion: {}", cmd.material_id.0);

        // 获取物料
        let mut material = self
            .material_repo
            .find_by_id(&cmd.material_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料不存在"))?;

        // 标记删除
        material.mark_for_deletion();

        // 更新审计信息
        {
            let audit = material.audit_info_mut();
            audit.updated_by = Some(cmd.user_id);
            audit.updated_at = chrono::Utc::now();
        }

        // 保存
        self.material_repo.update(&material).await?;

        info!("Material marked for deletion successfully");
        Ok(())
    }

    // ========== 查询视图数据 ==========

    /// 获取工厂数据
    pub async fn get_plant_data(&self, query: GetPlantDataQuery) -> AppResult<PlantData> {
        let data = self
            .material_repo
            .get_plant_data(&query.material_id, &query.plant, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("工厂数据不存在"))?;

        Ok(data)
    }

    /// 获取销售数据
    pub async fn get_sales_data(&self, query: GetSalesDataQuery) -> AppResult<SalesData> {
        let data = self
            .material_repo
            .get_sales_data(&query.material_id, &query.sales_org, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("销售数据不存在"))?;

        Ok(data)
    }

    /// 获取采购数据
    pub async fn get_purchase_data(&self, query: GetPurchaseDataQuery) -> AppResult<PurchaseData> {
        let data = self
            .material_repo
            .get_purchase_data(&query.material_id, &query.purchase_org, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("采购数据不存在"))?;

        Ok(data)
    }

    /// 获取仓储数据
    pub async fn get_storage_data(&self, query: GetStorageDataQuery) -> AppResult<StorageData> {
        let data = self
            .material_repo
            .get_storage_data(&query.material_id, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("仓储数据不存在"))?;

        Ok(data)
    }

    /// 获取会计数据
    pub async fn get_accounting_data(
        &self,
        query: GetAccountingDataQuery,
    ) -> AppResult<AccountingData> {
        let data = self
            .material_repo
            .get_accounting_data(&query.material_id, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("会计数据不存在"))?;

        Ok(data)
    }

    /// 获取质量数据
    pub async fn get_quality_data(&self, query: GetQualityDataQuery) -> AppResult<QualityData> {
        let data = self
            .material_repo
            .get_quality_data(&query.material_id, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("质量数据不存在"))?;

        Ok(data)
    }

    // ========== 物料组 CRUD ==========

    /// 创建物料组
    pub async fn create_material_group(
        &self,
        cmd: CreateMaterialGroupCommand,
    ) -> AppResult<MaterialGroupId> {
        info!(
            "Creating material group: {} for tenant: {}",
            cmd.code, cmd.tenant_id.0
        );

        // 1. 验证命令
        cmd.validate()?;

        // 2. 检查编码是否已存在
        let exists = self
            .group_repo
            .exists_by_code(&cmd.code, &cmd.tenant_id)
            .await?;

        if exists {
            return Err(AppError::conflict(format!("物料组编码 {} 已存在", cmd.code)));
        }

        // 3. 创建物料组实体
        let mut group = if cmd.parent_id.is_some() {
            // 如果有父级，需要先获取父级物料组
            let parent = self
                .group_repo
                .find_by_id(cmd.parent_id.as_ref().unwrap(), &cmd.tenant_id)
                .await?
                .ok_or_else(|| AppError::not_found("父级物料组不存在"))?;

            MaterialGroup::new_child(
                cmd.tenant_id.clone(),
                cmd.code.clone(),
                cmd.name.clone(),
                &parent,
            )?
        } else {
            MaterialGroup::new_root(
                cmd.tenant_id.clone(),
                cmd.code.clone(),
                cmd.name.clone(),
            )
        };

        // 设置审计信息
        {
            let audit = group.audit_info_mut();
            audit.created_by = Some(cmd.user_id);
            audit.created_at = chrono::Utc::now();
        }

        // 设置可选字段
        if let Some(localized) = cmd.localized_name {
            group = group.with_localized_name(localized);
        }

        let group_id = group.id().clone();

        // 4. 保存到数据库
        self.group_repo.save(&group).await?;

        info!("Material group created successfully: {}", group_id.0);
        Ok(group_id)
    }

    /// 获取物料组
    pub async fn get_material_group(&self, query: GetMaterialGroupQuery) -> AppResult<MaterialGroup> {
        info!(
            "Getting material group: {} for tenant: {}",
            query.group_id.0, query.tenant_id.0
        );

        let group = self
            .group_repo
            .find_by_id(&query.group_id, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料组不存在"))?;

        Ok(group)
    }

    /// 按编码获取物料组
    pub async fn get_material_group_by_code(
        &self,
        query: GetMaterialGroupByCodeQuery,
    ) -> AppResult<MaterialGroup> {
        info!(
            "Getting material group by code: {} for tenant: {}",
            query.code, query.tenant_id.0
        );

        let group = self
            .group_repo
            .find_by_code(&query.code, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料组不存在"))?;

        Ok(group)
    }

    /// 更新物料组
    pub async fn update_material_group(&self, cmd: UpdateMaterialGroupCommand) -> AppResult<()> {
        info!(
            "Updating material group: {} for tenant: {}",
            cmd.group_id.0, cmd.tenant_id.0
        );

        // 1. 验证命令
        cmd.validate()?;

        // 2. 获取现有物料组
        let mut group = self
            .group_repo
            .find_by_id(&cmd.group_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料组不存在"))?;

        // 3. 更新字段
        if let Some(localized) = cmd.localized_name {
            group = group.with_localized_name(localized);
        }

        // 更新审计信息
        {
            let audit = group.audit_info_mut();
            audit.updated_by = Some(cmd.user_id);
            audit.updated_at = chrono::Utc::now();
        }

        // 4. 保存更新
        self.group_repo.update(&group).await?;

        info!("Material group updated successfully: {}", cmd.group_id.0);
        Ok(())
    }

    /// 删除物料组
    pub async fn delete_material_group(&self, cmd: DeleteMaterialGroupCommand) -> AppResult<()> {
        info!(
            "Deleting material group: {} for tenant: {}",
            cmd.group_id.0, cmd.tenant_id.0
        );

        // 1. 检查物料组是否存在
        let group = self
            .group_repo
            .find_by_id(&cmd.group_id, &cmd.tenant_id)
            .await?;

        if group.is_none() {
            warn!("Material group not found: {}", cmd.group_id.0);
            return Err(AppError::not_found("物料组不存在"));
        }

        // 2. 执行删除
        self.group_repo
            .delete(&cmd.group_id, &cmd.tenant_id)
            .await?;

        info!("Material group deleted successfully: {}", cmd.group_id.0);
        Ok(())
    }

    /// 列表查询物料组
    pub async fn list_material_groups(
        &self,
        query: ListMaterialGroupsQuery,
    ) -> AppResult<common::types::PagedResult<MaterialGroup>> {
        info!(
            "Listing material groups for tenant: {}, page: {}, size: {}",
            query.tenant_id.0, query.pagination.page, query.pagination.page_size
        );

        let result = self
            .group_repo
            .list(&query.tenant_id, query.parent_id.as_ref(), query.pagination)
            .await?;

        info!("Found {} material groups", result.total);
        Ok(result)
    }

    /// 查询子物料组
    pub async fn find_children_groups(
        &self,
        query: FindChildrenGroupsQuery,
    ) -> AppResult<Vec<MaterialGroup>> {
        info!(
            "Finding children groups for parent: {}",
            query.parent_id.0
        );

        let children = self
            .group_repo
            .find_children(&query.parent_id, &query.tenant_id)
            .await?;

        info!("Found {} children groups", children.len());
        Ok(children)
    }

    // ========== 物料类型 CRUD ==========

    /// 创建物料类型
    pub async fn create_material_type(
        &self,
        cmd: CreateMaterialTypeCommand,
    ) -> AppResult<MaterialTypeId> {
        info!(
            "Creating material type: {} for tenant: {}",
            cmd.code, cmd.tenant_id.0
        );

        // 1. 验证命令
        cmd.validate()?;

        // 2. 检查编码是否已存在
        let exists = self
            .type_repo
            .exists_by_code(&cmd.code, &cmd.tenant_id)
            .await?;

        if exists {
            return Err(AppError::conflict(format!(
                "物料类型编码 {} 已存在",
                cmd.code
            )));
        }

        // 3. 创建物料类型实体
        let type_id = MaterialTypeId::new();
        let mut material_type = MaterialType::new(
            type_id.clone(),
            cmd.tenant_id.clone(),
            cmd.code.clone(),
            cmd.name.clone(),
        );

        // 设置审计信息
        {
            let audit = material_type.audit_info_mut();
            audit.created_by = Some(cmd.user_id);
            audit.created_at = chrono::Utc::now();
        }

        // 设置可选字段
        if let Some(localized) = cmd.localized_name {
            material_type = material_type.with_localized_name(localized);
        }
        material_type = material_type
            .with_quantity_update(cmd.quantity_update)
            .with_value_update(cmd.value_update)
            .with_internal_procurement(cmd.internal_procurement)
            .with_external_procurement(cmd.external_procurement);

        let type_id = material_type.id().clone();

        // 4. 保存到数据库
        self.type_repo.save(&material_type).await?;

        info!("Material type created successfully: {}", type_id.0);
        Ok(type_id)
    }

    /// 获取物料类型
    pub async fn get_material_type(&self, query: GetMaterialTypeQuery) -> AppResult<MaterialType> {
        info!(
            "Getting material type: {} for tenant: {}",
            query.type_id.0, query.tenant_id.0
        );

        let material_type = self
            .type_repo
            .find_by_id(&query.type_id, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料类型不存在"))?;

        Ok(material_type)
    }

    /// 按编码获取物料类型
    pub async fn get_material_type_by_code(
        &self,
        query: GetMaterialTypeByCodeQuery,
    ) -> AppResult<MaterialType> {
        info!(
            "Getting material type by code: {} for tenant: {}",
            query.code, query.tenant_id.0
        );

        let material_type = self
            .type_repo
            .find_by_code(&query.code, &query.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料类型不存在"))?;

        Ok(material_type)
    }

    /// 更新物料类型
    pub async fn update_material_type(&self, cmd: UpdateMaterialTypeCommand) -> AppResult<()> {
        info!(
            "Updating material type: {} for tenant: {}",
            cmd.type_id.0, cmd.tenant_id.0
        );

        // 1. 验证命令
        cmd.validate()?;

        // 2. 获取现有物料类型
        let mut material_type = self
            .type_repo
            .find_by_id(&cmd.type_id, &cmd.tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("物料类型不存在"))?;

        // 3. 更新字段
        if let Some(localized) = cmd.localized_name {
            material_type = material_type.with_localized_name(localized);
        }
        if let Some(quantity_update) = cmd.quantity_update {
            material_type = material_type.with_quantity_update(quantity_update);
        }
        if let Some(value_update) = cmd.value_update {
            material_type = material_type.with_value_update(value_update);
        }
        if let Some(internal_procurement) = cmd.internal_procurement {
            material_type = material_type.with_internal_procurement(internal_procurement);
        }
        if let Some(external_procurement) = cmd.external_procurement {
            material_type = material_type.with_external_procurement(external_procurement);
        }

        // 更新审计信息
        {
            let audit = material_type.audit_info_mut();
            audit.updated_by = Some(cmd.user_id);
            audit.updated_at = chrono::Utc::now();
        }

        // 4. 保存更新
        self.type_repo.update(&material_type).await?;

        info!("Material type updated successfully: {}", cmd.type_id.0);
        Ok(())
    }

    /// 列表查询物料类型
    pub async fn list_material_types(
        &self,
        query: ListMaterialTypesQuery,
    ) -> AppResult<common::types::PagedResult<MaterialType>> {
        info!(
            "Listing material types for tenant: {}, page: {}, size: {}",
            query.tenant_id.0, query.pagination.page, query.pagination.page_size
        );

        let result = self
            .type_repo
            .list(&query.tenant_id, query.pagination)
            .await?;

        info!("Found {} material types", result.total);
        Ok(result)
    }
}
