//! 物料类型实体

use common::types::{AuditInfo, TenantId};
use serde::{Deserialize, Serialize};

use crate::domain::enums::PriceControl;
use crate::domain::value_objects::{LocalizedText, MaterialTypeId};

/// 物料类型实体
///
/// 定义物料的类型及其控制参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialType {
    /// 物料类型 ID
    id: MaterialTypeId,
    /// 租户 ID
    tenant_id: TenantId,
    /// 类型编码（如: ROH, HALB, FERT, HAWA）
    code: String,
    /// 类型名称
    name: String,
    /// 多语言名称
    localized_name: LocalizedText,
    /// 描述
    description: String,

    // 控制参数
    /// 数量更新
    quantity_update: bool,
    /// 价值更新
    value_update: bool,
    /// 允许内部采购
    internal_procurement: bool,
    /// 允许外部采购
    external_procurement: bool,

    // 默认值
    /// 默认评估类
    default_valuation_class: String,
    /// 默认价格控制
    default_price_control: PriceControl,

    /// 审计信息
    audit_info: AuditInfo,
}

impl MaterialType {
    /// 创建新的物料类型
    pub fn new(
        tenant_id: TenantId,
        code: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id: MaterialTypeId::new(),
            tenant_id,
            code: code.into(),
            name: name.into(),
            localized_name: LocalizedText::default(),
            description: String::new(),
            quantity_update: true,
            value_update: true,
            internal_procurement: false,
            external_procurement: true,
            default_valuation_class: String::new(),
            default_price_control: PriceControl::default(),
            audit_info: AuditInfo::default(),
        }
    }

    // ========== Getters ==========

    pub fn id(&self) -> &MaterialTypeId {
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

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn quantity_update(&self) -> bool {
        self.quantity_update
    }

    pub fn value_update(&self) -> bool {
        self.value_update
    }

    pub fn internal_procurement(&self) -> bool {
        self.internal_procurement
    }

    pub fn external_procurement(&self) -> bool {
        self.external_procurement
    }

    pub fn default_valuation_class(&self) -> &str {
        &self.default_valuation_class
    }

    pub fn default_price_control(&self) -> PriceControl {
        self.default_price_control
    }

    pub fn audit_info(&self) -> &AuditInfo {
        &self.audit_info
    }

    // ========== Builder pattern setters ==========

    pub fn with_localized_name(mut self, name: LocalizedText) -> Self {
        self.localized_name = name;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_control_parameters(
        mut self,
        quantity_update: bool,
        value_update: bool,
        internal_procurement: bool,
        external_procurement: bool,
    ) -> Self {
        self.quantity_update = quantity_update;
        self.value_update = value_update;
        self.internal_procurement = internal_procurement;
        self.external_procurement = external_procurement;
        self
    }

    pub fn with_defaults(
        mut self,
        valuation_class: impl Into<String>,
        price_control: PriceControl,
    ) -> Self {
        self.default_valuation_class = valuation_class.into();
        self.default_price_control = price_control;
        self
    }

    // ========== Mutable setters ==========

    /// 更新基本信息
    pub fn update(&mut self, name: impl Into<String>, description: impl Into<String>) {
        self.name = name.into();
        self.description = description.into();
        self.audit_info.update(None);
    }

    /// 更新多语言名称
    pub fn update_localized_name(&mut self, name: LocalizedText) {
        self.localized_name = name;
        self.audit_info.update(None);
    }

    /// 设置控制参数
    pub fn set_control_parameters(
        &mut self,
        quantity_update: bool,
        value_update: bool,
        internal_procurement: bool,
        external_procurement: bool,
    ) {
        self.quantity_update = quantity_update;
        self.value_update = value_update;
        self.internal_procurement = internal_procurement;
        self.external_procurement = external_procurement;
        self.audit_info.update(None);
    }

    /// 设置默认值
    pub fn set_defaults(
        &mut self,
        valuation_class: impl Into<String>,
        price_control: PriceControl,
    ) {
        self.default_valuation_class = valuation_class.into();
        self.default_price_control = price_control;
        self.audit_info.update(None);
    }
}
