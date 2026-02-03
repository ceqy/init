//! Material commands

use common::types::{TenantId, UserId};
use errors::AppResult;

use crate::domain::value_objects::{
    LocalizedText, MaterialGroupId, MaterialId, MaterialTypeId,
};
use crate::domain::views::{
    AccountingData, PlantData, PurchaseData, QualityData, SalesData, StorageData,
};

/// 创建物料命令
#[derive(Debug, Clone)]
pub struct CreateMaterialCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub material_number: String,
    pub description: String,
    pub localized_description: Option<LocalizedText>,
    pub material_type_id: MaterialTypeId,
    pub material_group_id: Option<MaterialGroupId>,
    pub base_unit: String,
    // 尺寸和重量
    pub gross_weight: Option<f64>,
    pub net_weight: Option<f64>,
    pub weight_unit: Option<String>,
    pub volume: Option<f64>,
    pub volume_unit: Option<String>,
    pub length: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub dimension_unit: Option<String>,
    // 扩展属性
    pub custom_attributes: Option<serde_json::Value>,
}

impl CreateMaterialCommand {
    pub fn validate(&self) -> AppResult<()> {
        // 验证物料编号
        if self.material_number.is_empty() {
            return Err(errors::AppError::validation("物料编号不能为空"));
        }
        if self.material_number.len() > 40 {
            return Err(errors::AppError::validation("物料编号长度不能超过40个字符"));
        }

        // 验证描述
        if self.description.is_empty() {
            return Err(errors::AppError::validation("物料描述不能为空"));
        }
        if self.description.len() > 200 {
            return Err(errors::AppError::validation("物料描述长度不能超过200个字符"));
        }

        // 验证基本单位
        if self.base_unit.is_empty() {
            return Err(errors::AppError::validation("基本单位不能为空"));
        }

        Ok(())
    }
}

/// 更新物料命令
#[derive(Debug, Clone)]
pub struct UpdateMaterialCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub description: Option<String>,
    pub localized_description: Option<LocalizedText>,
    pub material_group_id: Option<MaterialGroupId>,
    pub base_unit: Option<String>,
    // 尺寸和重量
    pub gross_weight: Option<f64>,
    pub net_weight: Option<f64>,
    pub weight_unit: Option<String>,
    pub volume: Option<f64>,
    pub volume_unit: Option<String>,
    pub length: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub dimension_unit: Option<String>,
    // 扩展属性
    pub custom_attributes: Option<serde_json::Value>,
}

impl UpdateMaterialCommand {
    pub fn validate(&self) -> AppResult<()> {
        // 验证描述
        if let Some(desc) = &self.description {
            if desc.is_empty() {
                return Err(errors::AppError::validation("物料描述不能为空"));
            }
            if desc.len() > 200 {
                return Err(errors::AppError::validation("物料描述长度不能超过200个字符"));
            }
        }

        // 验证基本单位
        if let Some(unit) = &self.base_unit {
            if unit.is_empty() {
                return Err(errors::AppError::validation("基本单位不能为空"));
            }
        }

        Ok(())
    }
}

/// 删除物料命令
#[derive(Debug, Clone)]
pub struct DeleteMaterialCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

/// 扩展到工厂命令
#[derive(Debug, Clone)]
pub struct ExtendToPlantCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub plant_data: PlantData,
}

/// 更新工厂数据命令
#[derive(Debug, Clone)]
pub struct UpdatePlantDataCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub plant: String,
    pub plant_data: PlantData,
}

/// 扩展到销售命令
#[derive(Debug, Clone)]
pub struct ExtendToSalesCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub sales_data: SalesData,
}

/// 更新销售数据命令
#[derive(Debug, Clone)]
pub struct UpdateSalesDataCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub sales_org: String,
    pub sales_data: SalesData,
}

/// 扩展到采购命令
#[derive(Debug, Clone)]
pub struct ExtendToPurchaseCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub purchase_data: PurchaseData,
}

/// 更新采购数据命令
#[derive(Debug, Clone)]
pub struct UpdatePurchaseDataCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub purchase_org: String,
    pub purchase_data: PurchaseData,
}

/// 扩展到仓储命令
#[derive(Debug, Clone)]
pub struct ExtendToStorageCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub storage_data: StorageData,
}

/// 更新仓储数据命令
#[derive(Debug, Clone)]
pub struct UpdateStorageDataCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub storage_data: StorageData,
}

/// 扩展到会计命令
#[derive(Debug, Clone)]
pub struct ExtendToAccountingCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub accounting_data: AccountingData,
}

/// 更新会计数据命令
#[derive(Debug, Clone)]
pub struct UpdateAccountingDataCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub accounting_data: AccountingData,
}

/// 扩展到质量命令
#[derive(Debug, Clone)]
pub struct ExtendToQualityCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub quality_data: QualityData,
}

/// 更新质量数据命令
#[derive(Debug, Clone)]
pub struct UpdateQualityDataCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub quality_data: QualityData,
}

/// 激活物料命令
#[derive(Debug, Clone)]
pub struct ActivateMaterialCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

/// 停用物料命令
#[derive(Debug, Clone)]
pub struct DeactivateMaterialCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

/// 冻结物料命令
#[derive(Debug, Clone)]
pub struct BlockMaterialCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

/// 标记删除命令
#[derive(Debug, Clone)]
pub struct MarkForDeletionCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

/// 设置替代物料命令
#[derive(Debug, Clone)]
pub struct SetAlternativeMaterialCommand {
    pub material_id: MaterialId,
    pub alternative_material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub plant: Option<String>,
    pub priority: i32,
    pub valid_from: Option<chrono::DateTime<chrono::Utc>>,
    pub valid_to: Option<chrono::DateTime<chrono::Utc>>,
}

/// 移除替代物料命令
#[derive(Debug, Clone)]
pub struct RemoveAlternativeMaterialCommand {
    pub material_id: MaterialId,
    pub alternative_material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub plant: Option<String>,
}

/// 创建单位换算命令
#[derive(Debug, Clone)]
pub struct CreateUnitConversionCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub from_unit: String,
    pub to_unit: String,
    pub numerator: f64,
    pub denominator: f64,
    pub ean_upc: Option<String>,
}

/// 删除单位换算命令
#[derive(Debug, Clone)]
pub struct DeleteUnitConversionCommand {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub from_unit: String,
    pub to_unit: String,
}
