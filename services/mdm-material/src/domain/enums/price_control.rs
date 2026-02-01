//! 价格控制枚举

use serde::{Deserialize, Serialize};

/// 价格控制类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PriceControl {
    /// 未指定
    #[default]
    Unspecified,
    /// 标准价格 (S)
    Standard,
    /// 移动平均价 (V)
    MovingAverage,
}

impl PriceControl {
    /// 获取 SAP 代码
    pub fn sap_code(&self) -> &'static str {
        match self {
            PriceControl::Unspecified => "",
            PriceControl::Standard => "S",
            PriceControl::MovingAverage => "V",
        }
    }

    /// 从 SAP 代码创建
    pub fn from_sap_code(code: &str) -> Self {
        match code.to_uppercase().as_str() {
            "S" => PriceControl::Standard,
            "V" => PriceControl::MovingAverage,
            _ => PriceControl::Unspecified,
        }
    }
}

impl From<i32> for PriceControl {
    fn from(value: i32) -> Self {
        match value {
            1 => PriceControl::Standard,
            2 => PriceControl::MovingAverage,
            _ => PriceControl::Unspecified,
        }
    }
}

impl From<PriceControl> for i32 {
    fn from(pc: PriceControl) -> Self {
        match pc {
            PriceControl::Unspecified => 0,
            PriceControl::Standard => 1,
            PriceControl::MovingAverage => 2,
        }
    }
}
