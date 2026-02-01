//! 物料组实体

use common::types::{AuditInfo, TenantId};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{LocalizedText, MaterialGroupId};
use crate::error::{ServiceError, ServiceResult};

/// 物料组实体
///
/// 用于对物料进行分类的层级结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialGroup {
    /// 物料组 ID
    id: MaterialGroupId,
    /// 租户 ID
    tenant_id: TenantId,
    /// 物料组编码
    code: String,
    /// 物料组名称
    name: String,
    /// 多语言名称
    localized_name: LocalizedText,
    /// 父级 ID
    parent_id: Option<MaterialGroupId>,
    /// 层级（从 1 开始）
    level: i32,
    /// 路径（如: 01/0101/010101）
    path: String,
    /// 是否叶子节点
    is_leaf: bool,
    /// 审计信息
    audit_info: AuditInfo,
}

impl MaterialGroup {
    /// 创建根级物料组
    pub fn new_root(
        tenant_id: TenantId,
        code: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let code = code.into();
        Self {
            id: MaterialGroupId::new(),
            tenant_id,
            code: code.clone(),
            name: name.into(),
            localized_name: LocalizedText::default(),
            parent_id: None,
            level: 1,
            path: code,
            is_leaf: true,
            audit_info: AuditInfo::default(),
        }
    }

    /// 创建子级物料组
    pub fn new_child(
        tenant_id: TenantId,
        code: impl Into<String>,
        name: impl Into<String>,
        parent: &MaterialGroup,
    ) -> ServiceResult<Self> {
        if parent.tenant_id != tenant_id {
            return Err(ServiceError::InvalidInput(
                "父级物料组属于不同租户".to_string(),
            ));
        }

        let code = code.into();
        let path = format!("{}/{}", parent.path, code);

        Ok(Self {
            id: MaterialGroupId::new(),
            tenant_id,
            code: code.clone(),
            name: name.into(),
            localized_name: LocalizedText::default(),
            parent_id: Some(parent.id.clone()),
            level: parent.level + 1,
            path,
            is_leaf: true,
            audit_info: AuditInfo::default(),
        })
    }

    // ========== Getters ==========

    pub fn id(&self) -> &MaterialGroupId {
        &self.id
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn localized_name(&self) -> &LocalizedText {
        &self.localized_name
    }

    pub fn parent_id(&self) -> Option<&MaterialGroupId> {
        self.parent_id.as_ref()
    }

    pub fn level(&self) -> i32 {
        self.level
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    pub fn audit_info(&self) -> &AuditInfo {
        &self.audit_info
    }

    // ========== Setters ==========

    /// 设置多语言名称
    pub fn with_localized_name(mut self, name: LocalizedText) -> Self {
        self.localized_name = name;
        self
    }

    /// 更新名称
    pub fn update_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
        self.audit_info.update(None);
    }

    /// 更新多语言名称
    pub fn update_localized_name(&mut self, name: LocalizedText) {
        self.localized_name = name;
        self.audit_info.update(None);
    }

    /// 标记为非叶子节点（当添加子节点时调用）
    pub fn mark_as_non_leaf(&mut self) {
        if self.is_leaf {
            self.is_leaf = false;
            self.audit_info.update(None);
        }
    }

    /// 标记为叶子节点（当删除所有子节点时调用）
    pub fn mark_as_leaf(&mut self) {
        if !self.is_leaf {
            self.is_leaf = true;
            self.audit_info.update(None);
        }
    }
}
