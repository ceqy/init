//! Material queries

use common::types::{Pagination, TenantId};

use crate::domain::entities::MaterialFilter;
use crate::domain::value_objects::{MaterialId, MaterialNumber};

/// 获取物料查询
#[derive(Debug, Clone)]
pub struct GetMaterialQuery {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
}

/// 按编号获取物料查询
#[derive(Debug, Clone)]
pub struct GetMaterialByNumberQuery {
    pub material_number: MaterialNumber,
    pub tenant_id: TenantId,
}

/// 列表物料查询
#[derive(Debug, Clone)]
pub struct ListMaterialsQuery {
    pub tenant_id: TenantId,
    pub filter: MaterialFilter,
    pub pagination: Pagination,
}

/// 搜索物料查询
#[derive(Debug, Clone)]
pub struct SearchMaterialsQuery {
    pub tenant_id: TenantId,
    pub query: String,
    pub pagination: Pagination,
}

/// 获取工厂数据查询
#[derive(Debug, Clone)]
pub struct GetPlantDataQuery {
    pub material_id: MaterialId,
    pub plant: String,
    pub tenant_id: TenantId,
}

/// 获取销售数据查询
#[derive(Debug, Clone)]
pub struct GetSalesDataQuery {
    pub material_id: MaterialId,
    pub sales_org: String,
    pub tenant_id: TenantId,
}

/// 获取采购数据查询
#[derive(Debug, Clone)]
pub struct GetPurchaseDataQuery {
    pub material_id: MaterialId,
    pub purchase_org: String,
    pub tenant_id: TenantId,
}

/// 获取仓储数据查询
#[derive(Debug, Clone)]
pub struct GetStorageDataQuery {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
}

/// 获取会计数据查询
#[derive(Debug, Clone)]
pub struct GetAccountingDataQuery {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
}

/// 获取质量数据查询
#[derive(Debug, Clone)]
pub struct GetQualityDataQuery {
    pub material_id: MaterialId,
    pub tenant_id: TenantId,
}
