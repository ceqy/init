//! 物料领域事件

use chrono::{DateTime, Utc};
use common::types::TenantId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::enums::DataStatus;
use crate::domain::value_objects::{MaterialGroupId, MaterialId, MaterialNumber, MaterialTypeId};

/// 事件基础信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// 事件 ID
    pub event_id: Uuid,
    /// 事件发生时间
    pub occurred_at: DateTime<Utc>,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 操作用户 ID
    pub user_id: Option<String>,
}

impl EventMetadata {
    pub fn new(tenant_id: TenantId, user_id: Option<String>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            tenant_id,
            user_id,
        }
    }
}

/// 物料领域事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaterialEvent {
    /// 物料已创建
    Created(MaterialCreated),
    /// 物料已更新
    Updated(MaterialUpdated),
    /// 物料已激活
    Activated(MaterialActivated),
    /// 物料已停用
    Deactivated(MaterialDeactivated),
    /// 物料已冻结
    Blocked(MaterialBlocked),
    /// 物料已标记删除
    MarkedForDeletion(MaterialMarkedForDeletion),
    /// 物料已删除
    Deleted(MaterialDeleted),
    /// 物料扩展到工厂
    ExtendedToPlant(MaterialExtendedToPlant),
    /// 物料扩展到销售组织
    ExtendedToSalesOrg(MaterialExtendedToSalesOrg),
}

/// 物料已创建事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialCreated {
    pub metadata: EventMetadata,
    pub material_id: MaterialId,
    pub material_number: MaterialNumber,
    pub description: String,
    pub material_type_id: MaterialTypeId,
    pub material_type_code: String,
    pub material_group_id: Option<MaterialGroupId>,
    pub material_group_code: Option<String>,
    pub base_unit: String,
}

impl MaterialCreated {
    pub fn new(
        tenant_id: TenantId,
        user_id: Option<String>,
        material_id: MaterialId,
        material_number: MaterialNumber,
        description: String,
        material_type_id: MaterialTypeId,
        material_type_code: String,
        material_group_id: Option<MaterialGroupId>,
        material_group_code: Option<String>,
        base_unit: String,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(tenant_id, user_id),
            material_id,
            material_number,
            description,
            material_type_id,
            material_type_code,
            material_group_id,
            material_group_code,
            base_unit,
        }
    }
}

/// 物料已更新事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialUpdated {
    pub metadata: EventMetadata,
    pub material_id: MaterialId,
    pub material_number: MaterialNumber,
    pub changes: Vec<String>,
}

impl MaterialUpdated {
    pub fn new(
        tenant_id: TenantId,
        user_id: Option<String>,
        material_id: MaterialId,
        material_number: MaterialNumber,
        changes: Vec<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(tenant_id, user_id),
            material_id,
            material_number,
            changes,
        }
    }
}

/// 物料已激活事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialActivated {
    pub metadata: EventMetadata,
    pub material_id: MaterialId,
    pub material_number: MaterialNumber,
    pub previous_status: DataStatus,
}

impl MaterialActivated {
    pub fn new(
        tenant_id: TenantId,
        user_id: Option<String>,
        material_id: MaterialId,
        material_number: MaterialNumber,
        previous_status: DataStatus,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(tenant_id, user_id),
            material_id,
            material_number,
            previous_status,
        }
    }
}

/// 物料已停用事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDeactivated {
    pub metadata: EventMetadata,
    pub material_id: MaterialId,
    pub material_number: MaterialNumber,
    pub previous_status: DataStatus,
}

impl MaterialDeactivated {
    pub fn new(
        tenant_id: TenantId,
        user_id: Option<String>,
        material_id: MaterialId,
        material_number: MaterialNumber,
        previous_status: DataStatus,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(tenant_id, user_id),
            material_id,
            material_number,
            previous_status,
        }
    }
}

/// 物料已冻结事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialBlocked {
    pub metadata: EventMetadata,
    pub material_id: MaterialId,
    pub material_number: MaterialNumber,
    pub previous_status: DataStatus,
    pub reason: Option<String>,
}

impl MaterialBlocked {
    pub fn new(
        tenant_id: TenantId,
        user_id: Option<String>,
        material_id: MaterialId,
        material_number: MaterialNumber,
        previous_status: DataStatus,
        reason: Option<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(tenant_id, user_id),
            material_id,
            material_number,
            previous_status,
            reason,
        }
    }
}

/// 物料已标记删除事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialMarkedForDeletion {
    pub metadata: EventMetadata,
    pub material_id: MaterialId,
    pub material_number: MaterialNumber,
}

impl MaterialMarkedForDeletion {
    pub fn new(
        tenant_id: TenantId,
        user_id: Option<String>,
        material_id: MaterialId,
        material_number: MaterialNumber,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(tenant_id, user_id),
            material_id,
            material_number,
        }
    }
}

/// 物料已删除事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDeleted {
    pub metadata: EventMetadata,
    pub material_id: MaterialId,
    pub material_number: MaterialNumber,
}

impl MaterialDeleted {
    pub fn new(
        tenant_id: TenantId,
        user_id: Option<String>,
        material_id: MaterialId,
        material_number: MaterialNumber,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(tenant_id, user_id),
            material_id,
            material_number,
        }
    }
}

/// 物料扩展到工厂事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialExtendedToPlant {
    pub metadata: EventMetadata,
    pub material_id: MaterialId,
    pub material_number: MaterialNumber,
    pub plant: String,
}

impl MaterialExtendedToPlant {
    pub fn new(
        tenant_id: TenantId,
        user_id: Option<String>,
        material_id: MaterialId,
        material_number: MaterialNumber,
        plant: String,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(tenant_id, user_id),
            material_id,
            material_number,
            plant,
        }
    }
}

/// 物料扩展到销售组织事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialExtendedToSalesOrg {
    pub metadata: EventMetadata,
    pub material_id: MaterialId,
    pub material_number: MaterialNumber,
    pub sales_org: String,
    pub distribution_channel: String,
}

impl MaterialExtendedToSalesOrg {
    pub fn new(
        tenant_id: TenantId,
        user_id: Option<String>,
        material_id: MaterialId,
        material_number: MaterialNumber,
        sales_org: String,
        distribution_channel: String,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(tenant_id, user_id),
            material_id,
            material_number,
            sales_org,
            distribution_channel,
        }
    }
}
