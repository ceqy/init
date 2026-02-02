//! Material group and type commands

use common::types::{TenantId, UserId};
use errors::AppResult;

use crate::domain::value_objects::{LocalizedText, MaterialGroupId, MaterialTypeId};

// ========== 物料组命令 ==========

/// 创建物料组命令
#[derive(Debug, Clone)]
pub struct CreateMaterialGroupCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub code: String,
    pub name: String,
    pub localized_name: Option<LocalizedText>,
    pub parent_id: Option<MaterialGroupId>,
}

impl CreateMaterialGroupCommand {
    pub fn validate(&self) -> AppResult<()> {
        if self.code.is_empty() {
            return Err(errors::AppError::validation("物料组编码不能为空"));
        }
        if self.code.len() > 20 {
            return Err(errors::AppError::validation("物料组编码长度不能超过20个字符"));
        }
        if self.name.is_empty() {
            return Err(errors::AppError::validation("物料组名称不能为空"));
        }
        if self.name.len() > 100 {
            return Err(errors::AppError::validation("物料组名称长度不能超过100个字符"));
        }
        Ok(())
    }
}

/// 更新物料组命令
#[derive(Debug, Clone)]
pub struct UpdateMaterialGroupCommand {
    pub group_id: MaterialGroupId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub name: Option<String>,
    pub localized_name: Option<LocalizedText>,
}

impl UpdateMaterialGroupCommand {
    pub fn validate(&self) -> AppResult<()> {
        if let Some(name) = &self.name {
            if name.is_empty() {
                return Err(errors::AppError::validation("物料组名称不能为空"));
            }
            if name.len() > 100 {
                return Err(errors::AppError::validation("物料组名称长度不能超过100个字符"));
            }
        }
        Ok(())
    }
}

/// 删除物料组命令
#[derive(Debug, Clone)]
pub struct DeleteMaterialGroupCommand {
    pub group_id: MaterialGroupId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

// ========== 物料类型命令 ==========

/// 创建物料类型命令
#[derive(Debug, Clone)]
pub struct CreateMaterialTypeCommand {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub code: String,
    pub name: String,
    pub localized_name: Option<LocalizedText>,
    pub quantity_update: bool,
    pub value_update: bool,
    pub internal_procurement: bool,
    pub external_procurement: bool,
}

impl CreateMaterialTypeCommand {
    pub fn validate(&self) -> AppResult<()> {
        if self.code.is_empty() {
            return Err(errors::AppError::validation("物料类型编码不能为空"));
        }
        if self.code.len() > 10 {
            return Err(errors::AppError::validation("物料类型编码长度不能超过10个字符"));
        }
        if self.name.is_empty() {
            return Err(errors::AppError::validation("物料类型名称不能为空"));
        }
        if self.name.len() > 100 {
            return Err(errors::AppError::validation("物料类型名称长度不能超过100个字符"));
        }
        Ok(())
    }
}

/// 更新物料类型命令
#[derive(Debug, Clone)]
pub struct UpdateMaterialTypeCommand {
    pub type_id: MaterialTypeId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub name: Option<String>,
    pub localized_name: Option<LocalizedText>,
    pub quantity_update: Option<bool>,
    pub value_update: Option<bool>,
    pub internal_procurement: Option<bool>,
    pub external_procurement: Option<bool>,
}

impl UpdateMaterialTypeCommand {
    pub fn validate(&self) -> AppResult<()> {
        if let Some(name) = &self.name {
            if name.is_empty() {
                return Err(errors::AppError::validation("物料类型名称不能为空"));
            }
            if name.len() > 100 {
                return Err(errors::AppError::validation("物料类型名称长度不能超过100个字符"));
            }
        }
        Ok(())
    }
}
