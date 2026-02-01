//! 数据库行映射结构

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// 物料类型数据库行
#[derive(Debug, FromRow)]
pub struct MaterialTypeRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub code: String,
    pub name: String,
    pub localized_name: Option<serde_json::Value>,
    pub quantity_update: bool,
    pub value_update: bool,
    pub internal_procurement: bool,
    pub external_procurement: bool,
    pub default_valuation_class: Option<String>,
    pub default_price_control: i16,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// 物料组数据库行
#[derive(Debug, FromRow)]
pub struct MaterialGroupRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub code: String,
    pub name: String,
    pub localized_name: Option<serde_json::Value>,
    pub parent_id: Option<Uuid>,
    pub level: i32,
    pub path: String,
    pub is_leaf: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// 物料数据库行
#[derive(Debug, FromRow)]
pub struct MaterialRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub material_number: String,
    pub description: String,
    pub localized_description: Option<serde_json::Value>,
    pub material_type_id: Uuid,
    pub material_type_code: String,
    pub material_group_id: Option<Uuid>,
    pub material_group_code: Option<String>,
    pub base_unit: String,
    pub gross_weight: Option<rust_decimal::Decimal>,
    pub net_weight: Option<rust_decimal::Decimal>,
    pub weight_unit: Option<String>,
    pub volume: Option<rust_decimal::Decimal>,
    pub volume_unit: Option<String>,
    pub length: Option<rust_decimal::Decimal>,
    pub width: Option<rust_decimal::Decimal>,
    pub height: Option<rust_decimal::Decimal>,
    pub dimension_unit: Option<String>,
    pub status: i16,
    pub custom_attributes: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// 物料工厂视图数据库行
#[derive(Debug, FromRow)]
pub struct MaterialPlantDataRow {
    pub id: Uuid,
    pub material_id: Uuid,
    pub tenant_id: Uuid,
    pub plant: String,
    pub mrp_type: Option<String>,
    pub mrp_controller: Option<String>,
    pub reorder_point: Option<rust_decimal::Decimal>,
    pub safety_stock: Option<rust_decimal::Decimal>,
    pub lot_size: Option<String>,
    pub minimum_lot_size: Option<rust_decimal::Decimal>,
    pub maximum_lot_size: Option<rust_decimal::Decimal>,
    pub fixed_lot_size: Option<rust_decimal::Decimal>,
    pub rounding_value: Option<rust_decimal::Decimal>,
    pub planned_delivery_days: Option<i32>,
    pub gr_processing_days: Option<i32>,
    pub procurement_type: i16,
    pub special_procurement: Option<String>,
    pub production_storage_location: Option<String>,
    pub batch_management: bool,
    pub serial_number_profile: Option<String>,
    pub abc_indicator: Option<String>,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// 物料销售视图数据库行
#[derive(Debug, FromRow)]
pub struct MaterialSalesDataRow {
    pub id: Uuid,
    pub material_id: Uuid,
    pub tenant_id: Uuid,
    pub sales_org: String,
    pub distribution_channel: String,
    pub sales_unit: Option<String>,
    pub minimum_order_quantity: Option<rust_decimal::Decimal>,
    pub minimum_delivery_quantity: Option<rust_decimal::Decimal>,
    pub delivery_unit: Option<String>,
    pub delivery_unit_quantity: Option<rust_decimal::Decimal>,
    pub pricing_reference_material: Option<String>,
    pub item_category_group: Option<String>,
    pub account_assignment_group: Option<String>,
    pub tax_classification: Option<String>,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// 物料采购视图数据库行
#[derive(Debug, FromRow)]
pub struct MaterialPurchaseDataRow {
    pub id: Uuid,
    pub material_id: Uuid,
    pub tenant_id: Uuid,
    pub purchase_org: String,
    pub plant: Option<String>,
    pub purchase_unit: Option<String>,
    pub purchasing_group: Option<String>,
    pub order_unit: Option<String>,
    pub planned_delivery_days: Option<i32>,
    pub gr_processing_days: Option<i32>,
    pub under_delivery_tolerance: Option<rust_decimal::Decimal>,
    pub over_delivery_tolerance: Option<rust_decimal::Decimal>,
    pub unlimited_over_delivery: Option<bool>,
    pub preferred_vendor_id: Option<String>,
    pub standard_price_amount: Option<rust_decimal::Decimal>,
    pub standard_price_currency: Option<String>,
    pub price_unit: Option<rust_decimal::Decimal>,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// 物料仓储视图数据库行
#[derive(Debug, FromRow)]
pub struct MaterialStorageDataRow {
    pub id: Uuid,
    pub material_id: Uuid,
    pub tenant_id: Uuid,
    pub plant: String,
    pub storage_location: String,
    pub warehouse_number: Option<String>,
    pub storage_type: Option<String>,
    pub storage_bin: Option<String>,
    pub picking_area: Option<String>,
    pub max_storage_quantity: Option<rust_decimal::Decimal>,
    pub min_storage_quantity: Option<rust_decimal::Decimal>,
    pub replenishment_quantity: Option<rust_decimal::Decimal>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// 物料会计视图数据库行
#[derive(Debug, FromRow)]
pub struct MaterialAccountingDataRow {
    pub id: Uuid,
    pub material_id: Uuid,
    pub tenant_id: Uuid,
    pub plant: String,
    pub valuation_area: String,
    pub valuation_class: Option<String>,
    pub valuation_category: Option<String>,
    pub price_control: i16,
    pub standard_price_amount: Option<rust_decimal::Decimal>,
    pub standard_price_currency: Option<String>,
    pub moving_average_price_amount: Option<rust_decimal::Decimal>,
    pub moving_average_price_currency: Option<String>,
    pub price_unit: Option<rust_decimal::Decimal>,
    pub inventory_account: Option<String>,
    pub price_difference_account: Option<String>,
    pub cost_element: Option<String>,
    pub costing_lot_size: Option<rust_decimal::Decimal>,
    pub with_qty_structure: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// 物料质量视图数据库行
#[derive(Debug, FromRow)]
pub struct MaterialQualityDataRow {
    pub id: Uuid,
    pub material_id: Uuid,
    pub tenant_id: Uuid,
    pub plant: String,
    pub inspection_active: bool,
    pub inspection_type: Option<String>,
    pub inspection_interval: Option<i32>,
    pub sample_percentage: Option<rust_decimal::Decimal>,
    pub shelf_life_days: Option<i32>,
    pub remaining_shelf_life_days: Option<i32>,
    pub certificate_type: Option<String>,
    pub certificate_required: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// 物料单位换算数据库行
#[derive(Debug, FromRow)]
pub struct MaterialUnitConversionRow {
    pub id: Uuid,
    pub material_id: Uuid,
    pub tenant_id: Uuid,
    pub from_unit: String,
    pub to_unit: String,
    pub numerator: rust_decimal::Decimal,
    pub denominator: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// 替代物料数据库行
#[derive(Debug, FromRow)]
pub struct MaterialAlternativeRow {
    pub id: Uuid,
    pub material_id: Uuid,
    pub alternative_material_id: Uuid,
    pub tenant_id: Uuid,
    pub priority: i32,
    pub usage_probability: Option<rust_decimal::Decimal>,
    pub plant: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}
