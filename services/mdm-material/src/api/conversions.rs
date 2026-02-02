//! Proto to Domain conversions

use common::types::{Pagination, TenantId, UserId};
use errors::{AppError, AppResult};
use uuid::Uuid;

use crate::domain::value_objects::{MaterialGroupId, MaterialId, MaterialTypeId};

// 从 gRPC metadata 中提取租户 ID
pub fn extract_tenant_id(metadata: &tonic::metadata::MetadataMap) -> AppResult<TenantId> {
    let tenant_id_str = metadata
        .get("x-tenant-id")
        .ok_or_else(|| AppError::validation("Missing tenant ID in metadata"))?
        .to_str()
        .map_err(|_| AppError::validation("Invalid tenant ID format"))?;

    let uuid = Uuid::parse_str(tenant_id_str)
        .map_err(|_| AppError::validation("Invalid tenant ID UUID"))?;

    Ok(TenantId(uuid))
}

// 从 gRPC metadata 中提取用户 ID
pub fn extract_user_id(metadata: &tonic::metadata::MetadataMap) -> AppResult<UserId> {
    let user_id_str = metadata
        .get("x-user-id")
        .ok_or_else(|| AppError::validation("Missing user ID in metadata"))?
        .to_str()
        .map_err(|_| AppError::validation("Invalid user ID format"))?;

    let uuid = Uuid::parse_str(user_id_str)
        .map_err(|_| AppError::validation("Invalid user ID UUID"))?;

    Ok(UserId(uuid))
}

// 解析 UUID 字符串
pub fn parse_uuid(s: &str, field_name: &str) -> AppResult<Uuid> {
    Uuid::parse_str(s).map_err(|_| AppError::validation(format!("Invalid {} UUID", field_name)))
}

// 解析 MaterialId
pub fn parse_material_id(s: &str) -> AppResult<MaterialId> {
    Ok(MaterialId(parse_uuid(s, "material ID")?))
}

// 解析 MaterialGroupId
pub fn parse_material_group_id(s: &str) -> AppResult<MaterialGroupId> {
    Ok(MaterialGroupId(parse_uuid(s, "material group ID")?))
}

// 解析 MaterialTypeId
pub fn parse_material_type_id(s: &str) -> AppResult<MaterialTypeId> {
    Ok(MaterialTypeId(parse_uuid(s, "material type ID")?))
}

// 解析分页参数
pub fn parse_pagination(page: i32, page_size: i32) -> Pagination {
    let page = if page < 1 { 1 } else { page as u32 };
    let page_size = if page_size < 1 {
        20
    } else if page_size > 100 {
        100
    } else {
        page_size as u32
    };

    Pagination { page, page_size }
}
