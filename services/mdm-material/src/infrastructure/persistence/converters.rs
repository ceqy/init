//! 数据库行到领域对象的转换

use std::collections::HashMap;

use common::types::{AuditInfo, TenantId, UserId};
use uuid::Uuid;

use crate::domain::entities::{Material, MaterialGroup, MaterialType};
use crate::domain::enums::{
    DataStatus, PlantMaterialStatus, PriceControl, ProcurementType, PurchaseMaterialStatus,
    SalesMaterialStatus,
};
use crate::domain::value_objects::{
    AlternativeMaterial, LocalizedText, MaterialGroupId, MaterialId, MaterialNumber, MaterialTypeId,
    UnitConversion,
};
use crate::domain::views::{
    AccountingData, Money, PlantData, PurchaseData, QualityData, SalesData, StorageData,
};

use super::rows::{
    MaterialAccountingDataRow, MaterialAlternativeRow, MaterialGroupRow, MaterialPlantDataRow,
    MaterialPurchaseDataRow, MaterialQualityDataRow, MaterialRow, MaterialSalesDataRow,
    MaterialStorageDataRow, MaterialTypeRow, MaterialUnitConversionRow,
};

/// 将 MaterialTypeRow 转换为 MaterialType
pub fn material_type_from_row(row: MaterialTypeRow) -> MaterialType {
    let localized_name = row
        .localized_name
        .and_then(|v| serde_json::from_value::<HashMap<String, String>>(v).ok())
        .map(|translations| {
            let mut text = LocalizedText::new(row.name.clone());
            for (lang, value) in translations {
                text.set_translation(&lang, value);
            }
            text
        });

    let price_control = match row.default_price_control {
        1 => PriceControl::Standard,
        2 => PriceControl::MovingAverage,
        _ => PriceControl::Unspecified,
    };

    let audit_info = build_audit_info(
        row.created_at,
        row.created_by,
        row.updated_at,
        row.updated_by,
    );

    let mut material_type = MaterialType::new(
        MaterialTypeId::from_uuid(row.id),
        TenantId::from_uuid(row.tenant_id),
        row.code,
        row.name,
    );

    if let Some(localized) = localized_name {
        material_type = material_type.with_localized_name(localized);
    }

    material_type
        .with_quantity_update(row.quantity_update)
        .with_value_update(row.value_update)
        .with_internal_procurement(row.internal_procurement)
        .with_external_procurement(row.external_procurement)
        .with_default_valuation_class(row.default_valuation_class)
        .with_default_price_control(price_control)
        .with_audit_info(audit_info)
}

/// 将 MaterialGroupRow 转换为 MaterialGroup
pub fn material_group_from_row(row: MaterialGroupRow) -> MaterialGroup {
    let localized_name = row
        .localized_name
        .and_then(|v| serde_json::from_value::<HashMap<String, String>>(v).ok())
        .map(|translations| {
            let mut text = LocalizedText::new(row.name.clone());
            for (lang, value) in translations {
                text.set_translation(&lang, value);
            }
            text
        });

    let audit_info = build_audit_info(
        row.created_at,
        row.created_by,
        row.updated_at,
        row.updated_by,
    );

    MaterialGroup::from_parts(
        MaterialGroupId::from_uuid(row.id),
        TenantId::from_uuid(row.tenant_id),
        row.code,
        row.name,
        localized_name,
        row.parent_id.map(MaterialGroupId::from_uuid),
        row.level,
        row.path,
        row.is_leaf,
        audit_info,
    )
}

/// 将 LocalizedText 转换为 JSON
pub fn localized_text_to_json(text: &LocalizedText) -> serde_json::Value {
    let translations = text.translations();
    serde_json::to_value(translations).unwrap_or(serde_json::Value::Object(Default::default()))
}

/// 将 MaterialPlantDataRow 转换为 PlantData
pub fn plant_data_from_row(row: MaterialPlantDataRow) -> PlantData {
    PlantData::new(&row.plant)
        .with_mrp_type(row.mrp_type.unwrap_or_default())
        .with_mrp_controller(row.mrp_controller.unwrap_or_default())
        .with_reorder_point(decimal_to_f64(row.reorder_point))
        .with_safety_stock(decimal_to_f64(row.safety_stock))
        .with_lot_sizes(
            decimal_to_f64(row.minimum_lot_size),
            decimal_to_f64(row.maximum_lot_size),
            decimal_to_f64(row.fixed_lot_size),
        )
        .with_rounding_value(decimal_to_f64(row.rounding_value))
        .with_delivery_days(
            row.planned_delivery_days.unwrap_or(0),
            row.gr_processing_days.unwrap_or(0),
        )
        .with_procurement_type(ProcurementType::from(row.procurement_type as i32))
        .with_special_procurement(row.special_procurement.unwrap_or_default())
        .with_storage_location(row.production_storage_location.unwrap_or_default())
        .with_batch_management(row.batch_management)
        .with_serial_number_profile(row.serial_number_profile.is_some())
        .with_abc_indicator(row.abc_indicator.unwrap_or_default())
        .with_status(PlantMaterialStatus::from(row.status as i32))
}

/// 将 MaterialSalesDataRow 转换为 SalesData
pub fn sales_data_from_row(row: MaterialSalesDataRow) -> SalesData {
    SalesData::new(&row.sales_org, &row.distribution_channel)
        .with_sales_unit(row.sales_unit.unwrap_or_default())
        .with_minimum_order_quantity(decimal_to_f64(row.minimum_order_quantity))
        .with_minimum_delivery_quantity(decimal_to_f64(row.minimum_delivery_quantity))
        .with_delivery_unit(row.delivery_unit.unwrap_or_default())
        .with_pricing_reference_material(row.pricing_reference_material.unwrap_or_default())
        .with_account_assignment_group(row.account_assignment_group.unwrap_or_default())
        .with_tax_classification(row.tax_classification.unwrap_or_default())
        .with_status(SalesMaterialStatus::from(row.status as i32))
}

/// 将 MaterialPurchaseDataRow 转换为 PurchaseData
pub fn purchase_data_from_row(row: MaterialPurchaseDataRow) -> PurchaseData {
    let mut data = PurchaseData::new(&row.purchase_org)
        .with_plant(row.plant.unwrap_or_default())
        .with_purchase_unit(row.purchase_unit.unwrap_or_default())
        .with_purchasing_group(row.purchasing_group.unwrap_or_default())
        .with_planned_delivery_days(row.planned_delivery_days.unwrap_or(0))
        .with_delivery_tolerances(
            decimal_to_f64(row.over_delivery_tolerance),
            decimal_to_f64(row.under_delivery_tolerance),
            row.unlimited_over_delivery.unwrap_or(false),
        )
        .with_preferred_vendor_id(row.preferred_vendor_id.unwrap_or_default())
        .with_status(PurchaseMaterialStatus::from(row.status as i32));

    if let (Some(amount), Some(currency)) = (row.standard_price_amount, row.standard_price_currency)
    {
        data = data.with_standard_price(Money::new(currency, decimal_to_i64(amount), 2));
    }

    data
}

/// 将 MaterialStorageDataRow 转换为 StorageData
pub fn storage_data_from_row(row: MaterialStorageDataRow) -> StorageData {
    StorageData::new(&row.plant, &row.storage_location)
        .with_warehouse_number(row.warehouse_number.unwrap_or_default())
        .with_storage_type(row.storage_type.unwrap_or_default())
        .with_storage_bin(row.storage_bin.unwrap_or_default())
        .with_storage_quantities(
            decimal_to_f64(row.min_storage_quantity),
            decimal_to_f64(row.max_storage_quantity),
        )
        .with_picking_area(row.picking_area.unwrap_or_default())
}

/// 将 MaterialAccountingDataRow 转换为 AccountingData
pub fn accounting_data_from_row(row: MaterialAccountingDataRow) -> AccountingData {
    let mut data = AccountingData::new(&row.plant, &row.valuation_area)
        .with_valuation_class(row.valuation_class.unwrap_or_default())
        .with_price_control(PriceControl::from(row.price_control as i32))
        .with_inventory_account(row.inventory_account.unwrap_or_default())
        .with_price_difference_account(row.price_difference_account.unwrap_or_default())
        .with_cost_element(row.cost_element.unwrap_or_default())
        .with_costing_lot_size(decimal_to_string(row.costing_lot_size))
        .with_qty_structure(row.with_qty_structure.unwrap_or(false));

    if let (Some(amount), Some(currency)) = (row.standard_price_amount, row.standard_price_currency)
    {
        data = data.with_standard_price(Money::new(currency, decimal_to_i64(amount), 2));
    }

    if let (Some(amount), Some(currency)) = (
        row.moving_average_price_amount,
        row.moving_average_price_currency,
    ) {
        data = data.with_moving_average_price(Money::new(currency, decimal_to_i64(amount), 2));
    }

    data
}

/// 将 MaterialQualityDataRow 转换为 QualityData
pub fn quality_data_from_row(row: MaterialQualityDataRow) -> QualityData {
    QualityData::new(&row.plant)
        .with_inspection_active(row.inspection_active)
        .with_inspection_type(row.inspection_type.unwrap_or_default())
        .with_sample_percentage(decimal_to_f64(row.sample_percentage))
        .with_certificate_required(row.certificate_required.unwrap_or(false))
        .with_shelf_life(
            row.shelf_life_days.unwrap_or(0),
            row.remaining_shelf_life_days.unwrap_or(0),
            row.shelf_life_days.unwrap_or(0),
        )
}

/// 将 MaterialUnitConversionRow 转换为 UnitConversion
pub fn unit_conversion_from_row(row: MaterialUnitConversionRow) -> Option<UnitConversion> {
    UnitConversion::new(
        &row.from_unit,
        &row.to_unit,
        decimal_to_f64(Some(row.numerator)),
        decimal_to_f64(Some(row.denominator)),
    )
    .ok()
}

/// 将 MaterialAlternativeRow 转换为 AlternativeMaterial
pub fn alternative_material_from_row(
    row: MaterialAlternativeRow,
    material_number: String,
    description: String,
) -> AlternativeMaterial {
    AlternativeMaterial::new(
        MaterialId::from_uuid(row.alternative_material_id),
        material_number,
        description,
        row.priority,
    )
    .with_plant(row.plant.unwrap_or_default())
    .with_validity(row.valid_from, row.valid_to)
}

/// 从 MaterialRow 和视图数据构建 Material
pub fn material_from_parts(
    row: MaterialRow,
    plant_data: Vec<PlantData>,
    sales_data: Vec<SalesData>,
    purchase_data: Vec<PurchaseData>,
    storage_data: Vec<StorageData>,
    accounting_data: Vec<AccountingData>,
    quality_data: Vec<QualityData>,
    unit_conversions: Vec<UnitConversion>,
) -> Material {
    let localized_description = parse_localized_text(
        &row.description,
        row.localized_description,
    );

    let custom_attributes: HashMap<String, String> = row
        .custom_attributes
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let audit_info = build_audit_info(
        row.created_at,
        row.created_by,
        row.updated_at,
        row.updated_by,
    );

    let status = DataStatus::from(row.status as i32);

    // 使用 MaterialNumber::new_unchecked 因为数据库中的数据已经验证过
    let material_number = MaterialNumber::new(&row.material_number)
        .unwrap_or_else(|_| MaterialNumber::new("INVALID").unwrap());

    Material::from_parts(
        MaterialId::from_uuid(row.id),
        TenantId::from_uuid(row.tenant_id),
        material_number,
        row.description,
        localized_description,
        MaterialTypeId::from_uuid(row.material_type_id),
        row.material_type_code,
        row.material_group_id.map(MaterialGroupId::from_uuid),
        row.material_group_code.unwrap_or_default(),
        row.base_unit,
        decimal_to_f64(row.gross_weight),
        decimal_to_f64(row.net_weight),
        row.weight_unit.unwrap_or_default(),
        decimal_to_f64(row.volume),
        row.volume_unit.unwrap_or_default(),
        decimal_to_f64(row.length),
        decimal_to_f64(row.width),
        decimal_to_f64(row.height),
        row.dimension_unit.unwrap_or_default(),
        status,
        plant_data,
        sales_data,
        purchase_data,
        storage_data,
        accounting_data,
        quality_data,
        unit_conversions,
        custom_attributes,
        audit_info,
    )
}

/// 构建 AuditInfo
fn build_audit_info(
    created_at: chrono::DateTime<chrono::Utc>,
    created_by: Option<Uuid>,
    updated_at: chrono::DateTime<chrono::Utc>,
    updated_by: Option<Uuid>,
) -> AuditInfo {
    AuditInfo {
        created_at,
        created_by: created_by.map(UserId::from_uuid),
        updated_at,
        updated_by: updated_by.map(UserId::from_uuid),
    }
}

/// Decimal 转 f64
fn decimal_to_f64(value: Option<rust_decimal::Decimal>) -> f64 {
    value
        .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0)
}

/// Decimal 转 i64 (用于货币金额，假设2位小数)
fn decimal_to_i64(value: rust_decimal::Decimal) -> i64 {
    (value * rust_decimal::Decimal::from(100))
        .to_string()
        .parse::<i64>()
        .unwrap_or(0)
}

/// Decimal 转 String
fn decimal_to_string(value: Option<rust_decimal::Decimal>) -> String {
    value.map(|d| d.to_string()).unwrap_or_default()
}

/// 解析 LocalizedText 从 JSON
pub fn parse_localized_text(
    default_text: &str,
    json_value: Option<serde_json::Value>,
) -> LocalizedText {
    json_value
        .and_then(|v| serde_json::from_value::<HashMap<String, String>>(v).ok())
        .map(|translations| {
            let mut text = LocalizedText::new(default_text);
            for (lang, value) in translations {
                text.set_translation(&lang, value);
            }
            text
        })
        .unwrap_or_else(|| LocalizedText::new(default_text))
}
