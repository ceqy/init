//! 物料聚合根

use std::collections::HashMap;

use common::types::{AuditInfo, TenantId};
use domain_core::{AggregateRoot, Entity};
use errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};

use crate::domain::enums::DataStatus;
use crate::domain::value_objects::{
    LocalizedText, MaterialGroupId, MaterialId, MaterialNumber,
    MaterialTypeId, UnitConversion,
};
use crate::domain::views::{
    AccountingData, PlantData, PurchaseData, QualityData, SalesData, StorageData,
};

/// 物料聚合根
///
/// 物料主数据的核心实体，包含基本信息和多个视图扩展
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    /// 物料 ID
    id: MaterialId,
    /// 租户 ID
    tenant_id: TenantId,
    /// 物料编号（业务主键）
    material_number: MaterialNumber,
    /// 物料描述
    description: String,
    /// 多语言描述
    localized_description: LocalizedText,

    // 基本数据
    /// 物料类型 ID
    material_type_id: MaterialTypeId,
    /// 物料类型编码
    material_type_code: String,
    /// 物料组 ID
    material_group_id: Option<MaterialGroupId>,
    /// 物料组编码
    material_group_code: String,
    /// 基本计量单位
    base_unit: String,
    /// 旧物料编号
    old_material_number: String,
    /// 外部物料组
    external_material_group: String,
    /// 产品组
    division: String,

    // 尺寸/重量
    /// 毛重
    gross_weight: f64,
    /// 净重
    net_weight: f64,
    /// 重量单位
    weight_unit: String,
    /// 体积
    volume: f64,
    /// 体积单位
    volume_unit: String,
    /// 长度
    length: f64,
    /// 宽度
    width: f64,
    /// 高度
    height: f64,
    /// 尺寸单位
    dimension_unit: String,

    // 包装数据
    /// EAN/UPC 码
    ean_upc: String,
    /// EAN 类别
    ean_category: String,

    /// 物料状态
    status: DataStatus,
    /// 删除标记
    deletion_flag: bool,

    // 扩展视图
    /// 工厂视图
    plant_data: Vec<PlantData>,
    /// 销售视图
    sales_data: Vec<SalesData>,
    /// 采购视图
    purchase_data: Vec<PurchaseData>,
    /// 仓储视图
    storage_data: Vec<StorageData>,
    /// 会计视图
    accounting_data: Vec<AccountingData>,
    /// 质量视图
    quality_data: Vec<QualityData>,

    /// 单位换算
    unit_conversions: Vec<UnitConversion>,

    /// 自定义属性
    custom_attributes: HashMap<String, String>,

    /// 审计信息
    audit_info: AuditInfo,
}

impl Material {
    /// 创建新物料
    pub fn new(
        tenant_id: TenantId,
        material_number: MaterialNumber,
        description: impl Into<String>,
        material_type_id: MaterialTypeId,
        material_type_code: impl Into<String>,
        base_unit: impl Into<String>,
    ) -> Self {
        Self {
            id: MaterialId::new(),
            tenant_id,
            material_number,
            description: description.into(),
            localized_description: LocalizedText::default(),
            material_type_id,
            material_type_code: material_type_code.into(),
            material_group_id: None,
            material_group_code: String::new(),
            base_unit: base_unit.into(),
            old_material_number: String::new(),
            external_material_group: String::new(),
            division: String::new(),
            gross_weight: 0.0,
            net_weight: 0.0,
            weight_unit: String::new(),
            volume: 0.0,
            volume_unit: String::new(),
            length: 0.0,
            width: 0.0,
            height: 0.0,
            dimension_unit: String::new(),
            ean_upc: String::new(),
            ean_category: String::new(),
            status: DataStatus::Draft,
            deletion_flag: false,
            plant_data: Vec::new(),
            sales_data: Vec::new(),
            purchase_data: Vec::new(),
            storage_data: Vec::new(),
            accounting_data: Vec::new(),
            quality_data: Vec::new(),
            unit_conversions: Vec::new(),
            custom_attributes: HashMap::new(),
            audit_info: AuditInfo::default(),
        }
    }

    /// 从各部分构建物料（用于从数据库加载）
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        id: MaterialId,
        tenant_id: TenantId,
        material_number: MaterialNumber,
        description: String,
        localized_description: LocalizedText,
        material_type_id: MaterialTypeId,
        material_type_code: String,
        material_group_id: Option<MaterialGroupId>,
        material_group_code: String,
        base_unit: String,
        gross_weight: f64,
        net_weight: f64,
        weight_unit: String,
        volume: f64,
        volume_unit: String,
        length: f64,
        width: f64,
        height: f64,
        dimension_unit: String,
        status: DataStatus,
        plant_data: Vec<PlantData>,
        sales_data: Vec<SalesData>,
        purchase_data: Vec<PurchaseData>,
        storage_data: Vec<StorageData>,
        accounting_data: Vec<AccountingData>,
        quality_data: Vec<QualityData>,
        unit_conversions: Vec<UnitConversion>,
        custom_attributes: HashMap<String, String>,
        audit_info: AuditInfo,
    ) -> Self {
        Self {
            id,
            tenant_id,
            material_number,
            description,
            localized_description,
            material_type_id,
            material_type_code,
            material_group_id,
            material_group_code,
            base_unit,
            old_material_number: String::new(),
            external_material_group: String::new(),
            division: String::new(),
            gross_weight,
            net_weight,
            weight_unit,
            volume,
            volume_unit,
            length,
            width,
            height,
            dimension_unit,
            ean_upc: String::new(),
            ean_category: String::new(),
            status,
            deletion_flag: status == DataStatus::MarkedForDeletion,
            plant_data,
            sales_data,
            purchase_data,
            storage_data,
            accounting_data,
            quality_data,
            unit_conversions,
            custom_attributes,
            audit_info,
        }
    }

    // ========== Getters ==========

    pub fn id(&self) -> &MaterialId {
        &self.id
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    pub fn material_number(&self) -> &MaterialNumber {
        &self.material_number
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn localized_description(&self) -> &LocalizedText {
        &self.localized_description
    }

    pub fn material_type_id(&self) -> &MaterialTypeId {
        &self.material_type_id
    }

    pub fn material_type_code(&self) -> &str {
        &self.material_type_code
    }

    pub fn material_group_id(&self) -> Option<&MaterialGroupId> {
        self.material_group_id.as_ref()
    }

    pub fn material_group_code(&self) -> &str {
        &self.material_group_code
    }

    pub fn base_unit(&self) -> &str {
        &self.base_unit
    }

    pub fn old_material_number(&self) -> &str {
        &self.old_material_number
    }

    pub fn external_material_group(&self) -> &str {
        &self.external_material_group
    }

    pub fn division(&self) -> &str {
        &self.division
    }

    pub fn gross_weight(&self) -> f64 {
        self.gross_weight
    }

    pub fn net_weight(&self) -> f64 {
        self.net_weight
    }

    pub fn weight_unit(&self) -> &str {
        &self.weight_unit
    }

    pub fn volume(&self) -> f64 {
        self.volume
    }

    pub fn volume_unit(&self) -> &str {
        &self.volume_unit
    }

    pub fn length(&self) -> f64 {
        self.length
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn dimension_unit(&self) -> &str {
        &self.dimension_unit
    }

    pub fn ean_upc(&self) -> &str {
        &self.ean_upc
    }

    pub fn ean_category(&self) -> &str {
        &self.ean_category
    }

    pub fn status(&self) -> DataStatus {
        self.status
    }

    pub fn deletion_flag(&self) -> bool {
        self.deletion_flag
    }

    pub fn plant_data(&self) -> &[PlantData] {
        &self.plant_data
    }

    pub fn sales_data(&self) -> &[SalesData] {
        &self.sales_data
    }

    pub fn purchase_data(&self) -> &[PurchaseData] {
        &self.purchase_data
    }

    pub fn storage_data(&self) -> &[StorageData] {
        &self.storage_data
    }

    pub fn accounting_data(&self) -> &[AccountingData] {
        &self.accounting_data
    }

    pub fn quality_data(&self) -> &[QualityData] {
        &self.quality_data
    }

    pub fn unit_conversions(&self) -> &[UnitConversion] {
        &self.unit_conversions
    }

    pub fn custom_attributes(&self) -> &HashMap<String, String> {
        &self.custom_attributes
    }

    pub fn audit_info(&self) -> &AuditInfo {
        &self.audit_info
    }

    // ========== Setters (Builder pattern) ==========

    pub fn with_localized_description(mut self, description: LocalizedText) -> Self {
        self.localized_description = description;
        self
    }

    pub fn with_material_group(
        mut self,
        group_id: MaterialGroupId,
        group_code: impl Into<String>,
    ) -> Self {
        self.material_group_id = Some(group_id);
        self.material_group_code = group_code.into();
        self
    }

    pub fn with_old_material_number(mut self, number: impl Into<String>) -> Self {
        self.old_material_number = number.into();
        self
    }

    pub fn with_external_material_group(mut self, group: impl Into<String>) -> Self {
        self.external_material_group = group.into();
        self
    }

    pub fn with_division(mut self, division: impl Into<String>) -> Self {
        self.division = division.into();
        self
    }

    pub fn with_weight(mut self, gross: f64, net: f64, unit: impl Into<String>) -> Self {
        self.gross_weight = gross;
        self.net_weight = net;
        self.weight_unit = unit.into();
        self
    }

    pub fn with_volume(mut self, volume: f64, unit: impl Into<String>) -> Self {
        self.volume = volume;
        self.volume_unit = unit.into();
        self
    }

    pub fn with_dimensions(
        mut self,
        length: f64,
        width: f64,
        height: f64,
        unit: impl Into<String>,
    ) -> Self {
        self.length = length;
        self.width = width;
        self.height = height;
        self.dimension_unit = unit.into();
        self
    }

    pub fn with_ean(mut self, ean_upc: impl Into<String>, category: impl Into<String>) -> Self {
        self.ean_upc = ean_upc.into();
        self.ean_category = category.into();
        self
    }

    pub fn with_custom_attributes(mut self, attributes: HashMap<String, String>) -> Self {
        self.custom_attributes = attributes;
        self
    }

    // ========== 状态管理 ==========

    /// 激活物料
    pub fn activate(&mut self) -> AppResult<()> {
        if !self.status.can_activate() {
            return Err(AppError::validation(format!(
                "无法从 {:?} 状态激活物料",
                self.status
            )));
        }
        self.status = DataStatus::Active;
        self.audit_info.update(None);
        Ok(())
    }

    /// 停用物料
    pub fn deactivate(&mut self) -> AppResult<()> {
        if !self.status.can_deactivate() {
            return Err(AppError::validation(format!(
                "无法从 {:?} 状态停用物料",
                self.status
            )));
        }
        self.status = DataStatus::Inactive;
        self.audit_info.update(None);
        Ok(())
    }

    /// 冻结物料
    pub fn block(&mut self) -> AppResult<()> {
        if !self.status.can_block() {
            return Err(AppError::validation(format!(
                "无法从 {:?} 状态冻结物料",
                self.status
            )));
        }
        self.status = DataStatus::Blocked;
        self.audit_info.update(None);
        Ok(())
    }

    /// 标记删除
    pub fn mark_for_deletion(&mut self) -> AppResult<()> {
        if !self.status.can_mark_for_deletion() {
            return Err(AppError::validation(
                "物料已标记删除".to_string(),
            ));
        }
        self.status = DataStatus::MarkedForDeletion;
        self.deletion_flag = true;
        self.audit_info.update(None);
        Ok(())
    }

    // ========== 视图扩展 ==========

    /// 扩展到工厂
    pub fn extend_to_plant(&mut self, plant_data: PlantData) -> AppResult<()> {
        let plant = plant_data.plant().to_string();
        if self.plant_data.iter().any(|p| p.plant() == plant) {
            return Err(AppError::conflict(format!(
                "物料已扩展到工厂 {}",
                plant
            )));
        }
        self.plant_data.push(plant_data);
        self.audit_info.update(None);
        Ok(())
    }

    /// 扩展到销售组织
    pub fn extend_to_sales_org(&mut self, sales_data: SalesData) -> AppResult<()> {
        let key = sales_data.key();
        if self.sales_data.iter().any(|s| s.key() == key) {
            return Err(AppError::conflict(format!(
                "物料已扩展到销售组织 {}",
                key
            )));
        }
        self.sales_data.push(sales_data);
        self.audit_info.update(None);
        Ok(())
    }

    /// 扩展到采购组织
    pub fn extend_to_purchase_org(&mut self, purchase_data: PurchaseData) -> AppResult<()> {
        let key = purchase_data.key();
        if self.purchase_data.iter().any(|p| p.key() == key) {
            return Err(AppError::conflict(format!(
                "物料已扩展到采购组织 {}",
                key
            )));
        }
        self.purchase_data.push(purchase_data);
        self.audit_info.update(None);
        Ok(())
    }

    /// 扩展到仓储位置
    pub fn extend_to_storage(&mut self, storage_data: StorageData) -> AppResult<()> {
        let key = storage_data.key();
        if self.storage_data.iter().any(|s| s.key() == key) {
            return Err(AppError::conflict(format!(
                "物料已扩展到仓储位置 {}",
                key
            )));
        }
        self.storage_data.push(storage_data);
        self.audit_info.update(None);
        Ok(())
    }

    /// 扩展到会计视图
    pub fn extend_to_accounting(&mut self, accounting_data: AccountingData) -> AppResult<()> {
        let key = accounting_data.key();
        if self.accounting_data.iter().any(|a| a.key() == key) {
            return Err(AppError::conflict(format!(
                "物料已扩展到会计视图 {}",
                key
            )));
        }
        self.accounting_data.push(accounting_data);
        self.audit_info.update(None);
        Ok(())
    }

    /// 扩展到质量视图
    pub fn extend_to_quality(&mut self, quality_data: QualityData) -> AppResult<()> {
        let plant = quality_data.plant().to_string();
        if self.quality_data.iter().any(|q| q.plant() == plant) {
            return Err(AppError::conflict(format!(
                "物料已扩展到质量视图 {}",
                plant
            )));
        }
        self.quality_data.push(quality_data);
        self.audit_info.update(None);
        Ok(())
    }

    // ========== 视图更新 ==========

    /// 更新工厂数据
    pub fn update_plant_data(&mut self, plant: &str, data: PlantData) -> AppResult<()> {
        let pos = self
            .plant_data
            .iter()
            .position(|p| p.plant() == plant)
            .ok_or_else(|| AppError::not_found(format!("工厂 {} 不存在", plant)))?;
        self.plant_data[pos] = data;
        self.audit_info.update(None);
        Ok(())
    }

    /// 更新销售数据
    pub fn update_sales_data(
        &mut self,
        sales_org: &str,
        distribution_channel: &str,
        data: SalesData,
    ) -> AppResult<()> {
        let key = format!("{}_{}", sales_org, distribution_channel);
        let pos = self
            .sales_data
            .iter()
            .position(|s| s.key() == key)
            .ok_or_else(|| AppError::not_found(format!("销售组织 {} 不存在", key)))?;
        self.sales_data[pos] = data;
        self.audit_info.update(None);
        Ok(())
    }

    /// 更新采购数据
    pub fn update_purchase_data(&mut self, purchase_org: &str, data: PurchaseData) -> AppResult<()> {
        let pos = self
            .purchase_data
            .iter()
            .position(|p| p.purchase_org() == purchase_org)
            .ok_or_else(|| AppError::not_found(format!("采购组织 {} 不存在", purchase_org)))?;
        self.purchase_data[pos] = data;
        self.audit_info.update(None);
        Ok(())
    }

    // ========== 单位换算管理 ==========

    /// 添加单位换算
    pub fn add_unit_conversion(&mut self, conversion: UnitConversion) -> AppResult<()> {
        let from = conversion.source_unit();
        let to = conversion.target_unit();
        if self
            .unit_conversions
            .iter()
            .any(|c| c.source_unit() == from && c.target_unit() == to)
        {
            return Err(AppError::conflict(format!(
                "单位换算 {} -> {} 已存在",
                from, to
            )));
        }
        self.unit_conversions.push(conversion);
        self.audit_info.update(None);
        Ok(())
    }

    /// 移除单位换算
    pub fn remove_unit_conversion(&mut self, from_unit: &str, to_unit: &str) -> AppResult<()> {
        let pos = self
            .unit_conversions
            .iter()
            .position(|c| c.source_unit() == from_unit && c.target_unit() == to_unit)
            .ok_or_else(|| {
                AppError::not_found(format!("单位换算 {} -> {} 不存在", from_unit, to_unit))
            })?;
        self.unit_conversions.remove(pos);
        self.audit_info.update(None);
        Ok(())
    }

    // ========== 基本信息更新 ==========

    /// 更新描述
    pub fn update_description(&mut self, description: impl Into<String>) {
        self.description = description.into();
        self.audit_info.update(None);
    }

    /// 更新多语言描述
    pub fn update_localized_description(&mut self, description: LocalizedText) {
        self.localized_description = description;
        self.audit_info.update(None);
    }

    /// 更新物料组
    pub fn update_material_group(&mut self, group_id: MaterialGroupId, group_code: impl Into<String>) {
        self.material_group_id = Some(group_id);
        self.material_group_code = group_code.into();
        self.audit_info.update(None);
    }

    /// 设置自定义属性
    pub fn set_custom_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.custom_attributes.insert(key.into(), value.into());
        self.audit_info.update(None);
    }

    /// 移除自定义属性
    pub fn remove_custom_attribute(&mut self, key: &str) -> Option<String> {
        let result = self.custom_attributes.remove(key);
        if result.is_some() {
            self.audit_info.update(None);
        }
        result
    }

    // ========== 查询方法 ==========

    /// 获取指定工厂的数据
    pub fn get_plant_data(&self, plant: &str) -> Option<&PlantData> {
        self.plant_data.iter().find(|p| p.plant() == plant)
    }

    /// 获取指定销售组织的数据
    pub fn get_sales_data(&self, sales_org: &str, distribution_channel: &str) -> Option<&SalesData> {
        let key = format!("{}_{}", sales_org, distribution_channel);
        self.sales_data.iter().find(|s| s.key() == key)
    }

    /// 获取指定采购组织的数据
    pub fn get_purchase_data(&self, purchase_org: &str) -> Option<&PurchaseData> {
        self.purchase_data.iter().find(|p| p.purchase_org() == purchase_org)
    }

    /// 查找单位换算
    pub fn find_unit_conversion(&self, from_unit: &str, to_unit: &str) -> Option<&UnitConversion> {
        self.unit_conversions
            .iter()
            .find(|c| c.source_unit() == from_unit && c.target_unit() == to_unit)
    }
}

// ========== Entity/AggregateRoot trait 实现 ==========

impl Entity for Material {
    type Id = MaterialId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for Material {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit_info
    }

    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit_info
    }
}

/// 物料搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialSearchResult {
    /// 物料
    pub material: Material,
    /// 匹配分数
    pub score: f64,
    /// 高亮片段
    pub highlights: Vec<String>,
}

/// 物料过滤条件
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialFilter {
    /// 物料类型 ID
    pub material_type_id: Option<MaterialTypeId>,
    /// 物料组 ID
    pub material_group_id: Option<MaterialGroupId>,
    /// 状态
    pub status: Option<DataStatus>,
    /// 工厂
    pub plant: Option<String>,
    /// 搜索词
    pub search_term: Option<String>,
}
