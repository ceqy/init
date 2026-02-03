//! Domain to Proto conversions

use domain_core::{AggregateRoot, Entity};

use crate::domain::entities::{Material, MaterialGroup, MaterialType};
use crate::domain::value_objects::LocalizedText;

use crate::mdm_material::v1;
use crate::common;

// ========== Material 转换 ==========

pub fn material_to_proto(material: &Material) -> v1::Material {
    v1::Material {
        id: material.id().to_string(),
        material_number: material.material_number().as_str().to_string(),
        description: material.description().to_string(),
        localized_description: Some(localized_text_to_proto(material.localized_description())),
        material_type_id: material.material_type_id().to_string(),
        material_type_code: material.material_type_code().to_string(),
        material_group_id: material.material_group_id().map(|id| id.to_string()).unwrap_or_default(),
        material_group_code: material.material_group_code().to_string(),
        base_unit: material.base_unit().to_string(),
        old_material_number: material.old_material_number().to_string(),
        external_material_group: material.external_material_group().to_string(),
        division: material.division().to_string(),
        gross_weight: material.gross_weight(),
        net_weight: material.net_weight(),
        weight_unit: material.weight_unit().to_string(),
        volume: material.volume(),
        volume_unit: material.volume_unit().to_string(),
        length: material.length(),
        width: material.width(),
        height: material.height(),
        dimension_unit: material.dimension_unit().to_string(),
        ean_upc: material.ean_upc().to_string(),
        ean_category: material.ean_category().to_string(),
        status: material.status() as i32,
        deletion_flag: material.deletion_flag(),
        plant_data: vec![], // TODO: 实现视图数据转换
        sales_data: vec![],
        purchase_data: vec![],
        storage_data: vec![],
        accounting_data: vec![],
        quality_data: vec![],
        unit_conversions: vec![],
        custom_attributes: material.custom_attributes().clone(),
        attachments: vec![],
        audit_info: Some(audit_info_to_proto(material.audit_info())),
    }
}

// ========== MaterialGroup 转换 ==========

pub fn material_group_to_proto(group: &MaterialGroup) -> v1::MaterialGroup {
    v1::MaterialGroup {
        id: group.id().to_string(),
        code: group.code().to_string(),
        name: group.name().to_string(),
        localized_name: Some(localized_text_to_proto(group.localized_name())),
        parent_id: group.parent_id().map(|id| id.to_string()).unwrap_or_default(),
        parent_code: String::new(), // TODO: 实现父级编码查询
        level: group.level(),
        path: group.path().to_string(),
        is_leaf: group.is_leaf(),
        audit_info: Some(audit_info_to_proto(group.audit_info())),
    }
}

// ========== MaterialType 转换 ==========

pub fn material_type_to_proto(material_type: &MaterialType) -> v1::MaterialType {
    v1::MaterialType {
        id: material_type.id().to_string(),
        code: material_type.code().to_string(),
        name: material_type.name().to_string(),
        localized_name: Some(localized_text_to_proto(material_type.localized_name())),
        description: material_type.description().to_string(),
        quantity_update: material_type.quantity_update(),
        value_update: material_type.value_update(),
        internal_procurement: material_type.internal_procurement(),
        external_procurement: material_type.external_procurement(),
        default_valuation_class: material_type.default_valuation_class().to_string(),
        default_price_control: material_type.default_price_control() as i32,
        audit_info: Some(audit_info_to_proto(material_type.audit_info())),
    }
}

// ========== 辅助转换函数 ==========

fn localized_text_to_proto(text: &LocalizedText) -> common::v1::LocalizedText {
    common::v1::LocalizedText {
        default_text: text.default_text().to_string(),
        translations: text.translations().clone(),
    }
}

fn audit_info_to_proto(audit: &::common::types::AuditInfo) -> crate::common::v1::AuditInfo {
    common::v1::AuditInfo {
        created_at: Some(prost_types::Timestamp {
            seconds: audit.created_at.timestamp(),
            nanos: audit.created_at.timestamp_subsec_nanos() as i32,
        }),
        created_by: audit.created_by.as_ref().map(|u| u.0.to_string()).unwrap_or_default(),
        updated_at: Some(prost_types::Timestamp {
            seconds: audit.updated_at.timestamp(),
            nanos: audit.updated_at.timestamp_subsec_nanos() as i32,
        }),
        updated_by: audit.updated_by.as_ref().map(|u| u.0.to_string()).unwrap_or_default(),
    }
}

// ========== Proto to Domain 转换 ==========

impl From<common::v1::LocalizedText> for LocalizedText {
    fn from(proto: common::v1::LocalizedText) -> Self {
        // 使用第一个翻译作为默认文本，如果没有则使用空字符串
        let default_text = proto.translations.values().next().cloned().unwrap_or_default();
        LocalizedText::from_translations(default_text, proto.translations)
    }
}

// ========== 视图数据转换 ==========

use crate::domain::views::{
    PlantData, SalesData, PurchaseData, StorageData, AccountingData, QualityData,
};
use crate::domain::enums::{
    PlantMaterialStatus, SalesMaterialStatus, PurchaseMaterialStatus, PriceControl,
};

// ========== PlantData 转换 ==========

/// Proto PlantData 转换为 Domain PlantData
pub fn proto_to_plant_data(proto: v1::MaterialPlantData) -> PlantData {
    PlantData::new(proto.plant)
        .with_plant_name(proto.plant_name)
        .with_mrp_type(proto.mrp_type)
        .with_mrp_controller(proto.mrp_controller)
        .with_reorder_point(proto.reorder_point)
        .with_safety_stock(proto.safety_stock)
        .with_lot_sizes(
            proto.minimum_lot_size,
            proto.maximum_lot_size,
            proto.fixed_lot_size,
        )
        .with_rounding_value(proto.rounding_value)
        .with_delivery_days(proto.planned_delivery_days, proto.gr_processing_days)
        .with_procurement_type(proto_to_procurement_type(proto.procurement_type))
        .with_special_procurement(proto.special_procurement)
        .with_production_scheduler(proto.production_scheduler)
        .with_storage_location(proto.storage_location)
        .with_storage_bin(proto.storage_bin)
        .with_batch_management(proto.batch_management)
        .with_serial_number_profile(proto.serial_number_profile)
        .with_abc_indicator(proto.abc_indicator)
        .with_status(proto_to_plant_status(proto.status))
        .with_deletion_flag(proto.deletion_flag)
}

/// Domain PlantData 转换为 Proto PlantData
pub fn plant_data_to_proto(data: &PlantData) -> v1::MaterialPlantData {
    v1::MaterialPlantData {
        plant: data.plant().to_string(),
        plant_name: data.plant_name().to_string(),
        mrp_type: data.mrp_type().to_string(),
        mrp_controller: data.mrp_controller().to_string(),
        reorder_point: data.reorder_point(),
        safety_stock: data.safety_stock(),
        minimum_lot_size: data.minimum_lot_size(),
        maximum_lot_size: data.maximum_lot_size(),
        fixed_lot_size: data.fixed_lot_size(),
        rounding_value: data.rounding_value(),
        planned_delivery_days: data.planned_delivery_days(),
        gr_processing_days: data.gr_processing_days(),
        procurement_type: procurement_type_to_proto(data.procurement_type()) as i32,
        special_procurement: data.special_procurement().to_string(),
        production_scheduler: data.production_scheduler().to_string(),
        storage_location: data.storage_location().to_string(),
        storage_bin: data.storage_bin().to_string(),
        batch_management: data.batch_management(),
        serial_number_profile: data.serial_number_profile(),
        abc_indicator: data.abc_indicator().to_string(),
        status: plant_status_to_proto(data.status()) as i32,
        deletion_flag: data.deletion_flag(),
    }
}

// ========== 枚举转换辅助函数 ==========

fn proto_to_procurement_type(value: i32) -> crate::domain::enums::ProcurementType {
    use crate::domain::enums::ProcurementType;
    match value {
        0 => ProcurementType::Unspecified,
        1 => ProcurementType::External,
        2 => ProcurementType::Internal,
        3 => ProcurementType::Both,
        _ => ProcurementType::Unspecified, // 默认值
    }
}

fn procurement_type_to_proto(value: crate::domain::enums::ProcurementType) -> v1::ProcurementType {
    use crate::domain::enums::ProcurementType;
    match value {
        ProcurementType::Unspecified => v1::ProcurementType::Unspecified,
        ProcurementType::External => v1::ProcurementType::External,
        ProcurementType::Internal => v1::ProcurementType::Internal,
        ProcurementType::Both => v1::ProcurementType::Both,
    }
}

fn proto_to_plant_status(value: i32) -> PlantMaterialStatus {
    match value {
        0 => PlantMaterialStatus::Unspecified,
        1 => PlantMaterialStatus::Active,
        2 => PlantMaterialStatus::BlockedProcurement,
        3 => PlantMaterialStatus::BlockedProduction,
        4 => PlantMaterialStatus::BlockedAll,
        _ => PlantMaterialStatus::Unspecified, // 默认值
    }
}

fn plant_status_to_proto(value: PlantMaterialStatus) -> v1::PlantMaterialStatus {
    match value {
        PlantMaterialStatus::Unspecified => v1::PlantMaterialStatus::Unspecified,
        PlantMaterialStatus::Active => v1::PlantMaterialStatus::Active,
        PlantMaterialStatus::BlockedProcurement => v1::PlantMaterialStatus::BlockedProcurement,
        PlantMaterialStatus::BlockedProduction => v1::PlantMaterialStatus::BlockedProduction,
        PlantMaterialStatus::BlockedAll => v1::PlantMaterialStatus::BlockedAll,
    }
}

// ========== StorageData 转换 ==========

/// Proto StorageData 转换为 Domain StorageData
pub fn proto_to_storage_data(proto: v1::MaterialStorageData) -> StorageData {
    StorageData::new(proto.plant, proto.storage_location)
        .with_warehouse_number(proto.warehouse_number)
        .with_storage_type(proto.storage_type)
        .with_storage_bin(proto.storage_bin)
        .with_storage_quantities(proto.min_storage_quantity, proto.max_storage_quantity)
        .with_storage_unit_type(proto.storage_unit_type)
        .with_picking_area(proto.picking_area)
        .with_storage_section(proto.storage_section)
        .with_deletion_flag(proto.deletion_flag)
}

/// Domain StorageData 转换为 Proto StorageData
pub fn storage_data_to_proto(data: &StorageData) -> v1::MaterialStorageData {
    v1::MaterialStorageData {
        plant: data.plant().to_string(),
        storage_location: data.storage_location().to_string(),
        warehouse_number: data.warehouse_number().to_string(),
        storage_type: data.storage_type().to_string(),
        storage_bin: data.storage_bin().to_string(),
        max_storage_quantity: data.max_storage_quantity(),
        min_storage_quantity: data.min_storage_quantity(),
        storage_unit_type: data.storage_unit_type().to_string(),
        picking_area: data.picking_area().to_string(),
        storage_section: data.storage_section().to_string(),
        deletion_flag: data.deletion_flag(),
    }
}

// ========== QualityData 转换 ==========

/// Proto QualityData 转换为 Domain QualityData
pub fn proto_to_quality_data(proto: v1::MaterialQualityData) -> QualityData {
    QualityData::new(proto.plant)
        .with_inspection_active(proto.inspection_active)
        .with_inspection_type(proto.inspection_type)
        .with_inspection_plan(proto.inspection_plan)
        .with_sample_percentage(proto.sample_percentage)
        .with_certificate_required(proto.certificate_required)
        .with_quality_management_control_key(proto.quality_management_control_key)
        .with_shelf_life(
            proto.shelf_life_days,
            proto.remaining_shelf_life,
            proto.total_shelf_life,
        )
        .with_deletion_flag(proto.deletion_flag)
}

/// Domain QualityData 转换为 Proto QualityData
pub fn quality_data_to_proto(data: &QualityData) -> v1::MaterialQualityData {
    v1::MaterialQualityData {
        plant: data.plant().to_string(),
        inspection_active: data.inspection_active(),
        inspection_type: data.inspection_type().to_string(),
        inspection_plan: data.inspection_plan().to_string(),
        sample_percentage: data.sample_percentage(),
        certificate_required: data.certificate_required(),
        quality_management_control_key: data.quality_management_control_key().to_string(),
        shelf_life_days: data.shelf_life_days(),
        remaining_shelf_life: data.remaining_shelf_life(),
        total_shelf_life: data.total_shelf_life(),
        deletion_flag: data.deletion_flag(),
    }
}

// ========== SalesData 转换 ==========

/// Proto SalesData 转换为 Domain SalesData
pub fn proto_to_sales_data(proto: v1::MaterialSalesData) -> SalesData {
    SalesData::new(proto.sales_org, proto.distribution_channel)
        .with_division(proto.division)
        .with_sales_unit(proto.sales_unit)
        .with_minimum_order_quantity(proto.minimum_order_quantity)
        .with_minimum_delivery_quantity(proto.minimum_delivery_quantity)
        .with_delivery_unit(proto.delivery_unit)
        .with_delivery_days(proto.delivery_days)
        .with_pricing_reference_material(proto.pricing_reference_material)
        .with_material_pricing_group(proto.material_pricing_group)
        .with_account_assignment_group(proto.account_assignment_group)
        .with_tax_classification(proto.tax_classification)
        .with_availability_check(proto.availability_check)
        .with_status(proto_to_sales_status(proto.status))
        .with_deletion_flag(proto.deletion_flag)
}

/// Domain SalesData 转换为 Proto SalesData
pub fn sales_data_to_proto(data: &SalesData) -> v1::MaterialSalesData {
    v1::MaterialSalesData {
        sales_org: data.sales_org().to_string(),
        distribution_channel: data.distribution_channel().to_string(),
        division: data.division().to_string(),
        sales_unit: data.sales_unit().to_string(),
        minimum_order_quantity: data.minimum_order_quantity(),
        minimum_delivery_quantity: data.minimum_delivery_quantity(),
        delivery_unit: data.delivery_unit().to_string(),
        delivery_days: data.delivery_days(),
        pricing_reference_material: data.pricing_reference_material().to_string(),
        material_pricing_group: data.material_pricing_group().to_string(),
        account_assignment_group: data.account_assignment_group().to_string(),
        tax_classification: data.tax_classification().to_string(),
        availability_check: data.availability_check().to_string(),
        status: sales_status_to_proto(data.status()) as i32,
        deletion_flag: data.deletion_flag(),
    }
}

fn proto_to_sales_status(value: i32) -> SalesMaterialStatus {
    match value {
        0 => SalesMaterialStatus::Unspecified,
        1 => SalesMaterialStatus::Active,
        2 => SalesMaterialStatus::Blocked,
        _ => SalesMaterialStatus::Unspecified, // 默认值
    }
}

fn sales_status_to_proto(value: SalesMaterialStatus) -> v1::SalesMaterialStatus {
    match value {
        SalesMaterialStatus::Unspecified => v1::SalesMaterialStatus::Unspecified,
        SalesMaterialStatus::Active => v1::SalesMaterialStatus::Active,
        SalesMaterialStatus::Blocked => v1::SalesMaterialStatus::Blocked,
    }
}

// ========== 通用辅助函数 ==========

use crate::domain::views::Money;
use chrono::{DateTime, Utc};

/// Proto Money 转换为 Domain Money
fn proto_to_money(proto: Option<common::v1::Money>) -> Option<Money> {
    proto.map(|m| Money::new(m.currency, m.amount, m.decimal_places))
}

/// Domain Money 转换为 Proto Money
fn money_to_proto(money: &Option<Money>) -> Option<common::v1::Money> {
    money.as_ref().map(|m| common::v1::Money {
        currency: m.currency.clone(),
        amount: m.amount,
        decimal_places: m.decimal_places,
    })
}

/// Proto Timestamp 转换为 Domain DateTime
pub fn proto_to_timestamp(proto: Option<prost_types::Timestamp>) -> Option<DateTime<Utc>> {
    proto.and_then(|ts| {
        DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
    })
}

/// Domain DateTime 转换为 Proto Timestamp
fn timestamp_to_proto(dt: &Option<DateTime<Utc>>) -> Option<prost_types::Timestamp> {
    dt.as_ref().map(|d| prost_types::Timestamp {
        seconds: d.timestamp(),
        nanos: d.timestamp_subsec_nanos() as i32,
    })
}

// ========== AccountingData 转换 ==========

/// Proto AccountingData 转换为 Domain AccountingData
pub fn proto_to_accounting_data(proto: v1::MaterialAccountingData) -> AccountingData {
    let mut data = AccountingData::new(proto.plant, proto.valuation_area)
        .with_valuation_class(proto.valuation_class)
        .with_price_control(proto_to_price_control(proto.price_control))
        .with_price_unit(proto.price_unit, proto.price_unit_quantity)
        .with_inventory_account(proto.inventory_account)
        .with_price_difference_account(proto.price_difference_account)
        .with_cost_element(proto.cost_element)
        .with_costing_lot_size(proto.costing_lot_size)
        .with_qty_structure(proto.with_qty_structure)
        .with_deletion_flag(proto.deletion_flag);

    // 处理 Optional Money 字段
    if let Some(price) = proto_to_money(proto.standard_price) {
        data = data.with_standard_price(price);
    }
    if let Some(price) = proto_to_money(proto.moving_average_price) {
        data = data.with_moving_average_price(price);
    }

    data
}

/// Domain AccountingData 转换为 Proto AccountingData
pub fn accounting_data_to_proto(data: &AccountingData) -> v1::MaterialAccountingData {
    v1::MaterialAccountingData {
        plant: data.plant().to_string(),
        valuation_area: data.valuation_area().to_string(),
        valuation_class: data.valuation_class().to_string(),
        price_control: price_control_to_proto(data.price_control()) as i32,
        standard_price: money_to_proto_from_option_ref(data.standard_price()),
        moving_average_price: money_to_proto_from_option_ref(data.moving_average_price()),
        price_unit: data.price_unit().to_string(),
        price_unit_quantity: data.price_unit_quantity(),
        inventory_account: data.inventory_account().to_string(),
        price_difference_account: data.price_difference_account().to_string(),
        cost_element: data.cost_element().to_string(),
        costing_lot_size: data.costing_lot_size().to_string(),
        with_qty_structure: data.has_qty_structure(),
        deletion_flag: data.deletion_flag(),
    }
}

fn proto_to_price_control(value: i32) -> PriceControl {
    match value {
        0 => PriceControl::Unspecified,
        1 => PriceControl::Standard,
        2 => PriceControl::MovingAverage,
        _ => PriceControl::Unspecified, // 默认值
    }
}

fn price_control_to_proto(value: PriceControl) -> v1::PriceControl {
    match value {
        PriceControl::Unspecified => v1::PriceControl::Unspecified,
        PriceControl::Standard => v1::PriceControl::Standard,
        PriceControl::MovingAverage => v1::PriceControl::MovingAverage,
    }
}
// 修复 money_to_proto 函数
fn money_to_proto_from_option_ref(money: Option<&Money>) -> Option<common::v1::Money> {
    money.map(|m| common::v1::Money {
        currency: m.currency.clone(),
        amount: m.amount,
        decimal_places: m.decimal_places,
    })
}

// ========== PurchaseData 转换 ==========

/// Proto PurchaseData 转换为 Domain PurchaseData
pub fn proto_to_purchase_data(proto: v1::MaterialPurchaseData) -> PurchaseData {
    let mut data = PurchaseData::new(proto.purchase_org)
        .with_plant(proto.plant)
        .with_purchase_unit(proto.purchase_unit)
        .with_order_unit_conversion(proto.order_unit_conversion)
        .with_purchasing_group(proto.purchasing_group)
        .with_planned_delivery_days(proto.planned_delivery_days)
        .with_delivery_tolerances(
            proto.over_delivery_tolerance,
            proto.under_delivery_tolerance,
            proto.unlimited_over_delivery,
        )
        .with_preferred_vendor_id(proto.preferred_vendor_id)
        .with_automatic_po(proto.automatic_po)
        .with_source_list(proto.source_list)
        .with_status(proto_to_purchase_status(proto.status))
        .with_deletion_flag(proto.deletion_flag);

    // 处理 Optional Money 字段
    if let Some(price) = proto_to_money(proto.standard_price) {
        data = data.with_standard_price(price);
    }

    // 处理最近采购价和日期（需要同时存在）
    if let (Some(price), Some(date)) = (
        proto_to_money(proto.last_purchase_price),
        proto_to_timestamp(proto.last_purchase_date),
    ) {
        data = data.with_last_purchase_price(price, date);
    }

    data
}

/// Domain PurchaseData 转换为 Proto PurchaseData
pub fn purchase_data_to_proto(data: &PurchaseData) -> v1::MaterialPurchaseData {
    v1::MaterialPurchaseData {
        purchase_org: data.purchase_org().to_string(),
        plant: data.plant().to_string(),
        purchase_unit: data.purchase_unit().to_string(),
        order_unit_conversion: data.order_unit_conversion(),
        purchasing_group: data.purchasing_group().to_string(),
        planned_delivery_days: data.planned_delivery_days(),
        over_delivery_tolerance: data.over_delivery_tolerance(),
        under_delivery_tolerance: data.under_delivery_tolerance(),
        unlimited_over_delivery: data.unlimited_over_delivery(),
        preferred_vendor_id: data.preferred_vendor_id().to_string(),
        standard_price: money_to_proto_from_option_ref(data.standard_price()),
        last_purchase_price: money_to_proto_from_option_ref(data.last_purchase_price()),
        last_purchase_date: timestamp_to_proto(&data.last_purchase_date()),
        automatic_po: data.automatic_po(),
        source_list: data.source_list().to_string(),
        status: purchase_status_to_proto(data.status()) as i32,
        deletion_flag: data.deletion_flag(),
    }
}

fn proto_to_purchase_status(value: i32) -> PurchaseMaterialStatus {
    match value {
        0 => PurchaseMaterialStatus::Unspecified,
        1 => PurchaseMaterialStatus::Active,
        2 => PurchaseMaterialStatus::Blocked,
        _ => PurchaseMaterialStatus::Unspecified, // 默认值
    }
}

fn purchase_status_to_proto(value: PurchaseMaterialStatus) -> v1::PurchaseMaterialStatus {
    match value {
        PurchaseMaterialStatus::Unspecified => v1::PurchaseMaterialStatus::Unspecified,
        PurchaseMaterialStatus::Active => v1::PurchaseMaterialStatus::Active,
        PurchaseMaterialStatus::Blocked => v1::PurchaseMaterialStatus::Blocked,
    }
}

// ========== AlternativeMaterial 转换 ==========

/// Domain AlternativeMaterial 转换为 Proto AlternativeMaterial
pub fn alternative_material_to_proto(
    alt: &crate::domain::value_objects::AlternativeMaterial,
) -> v1::AlternativeMaterial {
    v1::AlternativeMaterial {
        material_id: alt.material_id().to_string(),
        material_number: alt.material_number().to_string(),
        description: alt.description().to_string(),
        plant: alt.plant().unwrap_or("").to_string(),
        priority: alt.priority(),
        validity: Some(common::v1::ValidityPeriod {
            valid_from: timestamp_to_proto(&alt.valid_from()),
            valid_to: timestamp_to_proto(&alt.valid_to()),
        }),
    }
}

// ========== UnitConversion 转换 ==========

/// Proto UnitConversion 转换为 Domain UnitConversion
pub fn proto_to_unit_conversion(proto: v1::UnitConversion) -> Option<crate::domain::value_objects::UnitConversion> {
    let mut conversion = crate::domain::value_objects::UnitConversion::new(
        proto.from_unit,
        proto.to_unit,
        proto.numerator,
        proto.denominator,
    ).ok()?;

    if !proto.ean_upc.is_empty() {
        conversion = conversion.with_ean_upc(proto.ean_upc);
    }

    Some(conversion)
}

/// Domain UnitConversion 转换为 Proto UnitConversion
pub fn unit_conversion_to_proto(
    conversion: &crate::domain::value_objects::UnitConversion,
) -> v1::UnitConversion {
    v1::UnitConversion {
        from_unit: conversion.source_unit().to_string(),
        to_unit: conversion.target_unit().to_string(),
        numerator: conversion.numerator(),
        denominator: conversion.denominator(),
        ean_upc: conversion.ean_upc().unwrap_or("").to_string(),
    }
}
