//! Material group and type queries

use common::types::{Pagination, TenantId};

use crate::domain::value_objects::{MaterialGroupId, MaterialTypeId};

// ========== 物料组查询 ==========

/// 获取物料组查询
#[derive(Debug, Clone)]
pub struct GetMaterialGroupQuery {
    pub group_id: MaterialGroupId,
    pub tenant_id: TenantId,
}

/// 按编码获取物料组查询
#[derive(Debug, Clone)]
pub struct GetMaterialGroupByCodeQuery {
    pub code: String,
    pub tenant_id: TenantId,
}

/// 列表物料组查询
#[derive(Debug, Clone)]
pub struct ListMaterialGroupsQuery {
    pub tenant_id: TenantId,
    pub parent_id: Option<MaterialGroupId>,
    pub pagination: Pagination,
}

/// 查询子物料组
#[derive(Debug, Clone)]
pub struct FindChildrenGroupsQuery {
    pub parent_id: MaterialGroupId,
    pub tenant_id: TenantId,
}

// ========== 物料类型查询 ==========

/// 获取物料类型查询
#[derive(Debug, Clone)]
pub struct GetMaterialTypeQuery {
    pub type_id: MaterialTypeId,
    pub tenant_id: TenantId,
}

/// 按编码获取物料类型查询
#[derive(Debug, Clone)]
pub struct GetMaterialTypeByCodeQuery {
    pub code: String,
    pub tenant_id: TenantId,
}

/// 列表物料类型查询
#[derive(Debug, Clone)]
pub struct ListMaterialTypesQuery {
    pub tenant_id: TenantId,
    pub pagination: Pagination,
}
