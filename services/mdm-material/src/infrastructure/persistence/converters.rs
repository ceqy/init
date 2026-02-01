//! 数据库行到领域对象的转换

use common::types::{AuditInfo, TenantId, UserId};
use uuid::Uuid;

use crate::domain::entities::{MaterialGroup, MaterialType};
use crate::domain::enums::PriceControl;
use crate::domain::value_objects::{LocalizedText, MaterialGroupId, MaterialTypeId};

use super::rows::{MaterialGroupRow, MaterialTypeRow};

/// 将 MaterialTypeRow 转换为 MaterialType
pub fn material_type_from_row(row: MaterialTypeRow) -> MaterialType {
    let localized_name = row
        .localized_name
        .and_then(|v| serde_json::from_value::<std::collections::HashMap<String, String>>(v).ok())
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
        .and_then(|v| serde_json::from_value::<std::collections::HashMap<String, String>>(v).ok())
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
