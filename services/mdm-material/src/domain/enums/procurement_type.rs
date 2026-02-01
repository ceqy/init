//! 采购类型枚举

use serde::{Deserialize, Serialize};

/// 采购类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProcurementType {
    /// 未指定
    #[default]
    Unspecified,
    /// 外部采购 (E)
    External,
    /// 内部生产 (F)
    Internal,
    /// 两者皆可 (X)
    Both,
}

impl ProcurementType {
    /// 获取 SAP 代码
    pub fn sap_code(&self) -> &'static str {
        match self {
            ProcurementType::Unspecified => "",
            ProcurementType::External => "E",
            ProcurementType::Internal => "F",
            ProcurementType::Both => "X",
        }
    }

    /// 从 SAP 代码创建
    pub fn from_sap_code(code: &str) -> Self {
        match code.to_uppercase().as_str() {
            "E" => ProcurementType::External,
            "F" => ProcurementType::Internal,
            "X" => ProcurementType::Both,
            _ => ProcurementType::Unspecified,
        }
    }

    /// 是否允许外部采购
    pub fn allows_external(&self) -> bool {
        matches!(self, ProcurementType::External | ProcurementType::Both)
    }

    /// 是否允许内部生产
    pub fn allows_internal(&self) -> bool {
        matches!(self, ProcurementType::Internal | ProcurementType::Both)
    }
}

impl From<i32> for ProcurementType {
    fn from(value: i32) -> Self {
        match value {
            1 => ProcurementType::External,
            2 => ProcurementType::Internal,
            3 => ProcurementType::Both,
            _ => ProcurementType::Unspecified,
        }
    }
}

impl From<ProcurementType> for i32 {
    fn from(pt: ProcurementType) -> Self {
        match pt {
            ProcurementType::Unspecified => 0,
            ProcurementType::External => 1,
            ProcurementType::Internal => 2,
            ProcurementType::Both => 3,
        }
    }
}
