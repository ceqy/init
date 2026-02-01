//! 单位换算值对象

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 单位换算错误
#[derive(Debug, Error)]
pub enum UnitConversionError {
    #[error("源单位不能为空")]
    EmptyFromUnit,
    #[error("目标单位不能为空")]
    EmptyToUnit,
    #[error("分子必须大于 0")]
    InvalidNumerator,
    #[error("分母必须大于 0")]
    InvalidDenominator,
    #[error("源单位和目标单位不能相同")]
    SameUnit,
}

/// 单位换算值对象
///
/// 表示从一个计量单位到另一个计量单位的换算关系
/// 换算公式: to_unit = from_unit * (numerator / denominator)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitConversion {
    /// 源单位
    from_unit: String,
    /// 目标单位
    to_unit: String,
    /// 分子
    numerator: f64,
    /// 分母
    denominator: f64,
    /// EAN/UPC 码（可选）
    ean_upc: Option<String>,
}

impl UnitConversion {
    /// 创建新的单位换算
    pub fn new(
        from_unit: impl Into<String>,
        to_unit: impl Into<String>,
        numerator: f64,
        denominator: f64,
    ) -> Result<Self, UnitConversionError> {
        let from_unit = from_unit.into().trim().to_uppercase();
        let to_unit = to_unit.into().trim().to_uppercase();

        if from_unit.is_empty() {
            return Err(UnitConversionError::EmptyFromUnit);
        }

        if to_unit.is_empty() {
            return Err(UnitConversionError::EmptyToUnit);
        }

        if from_unit == to_unit {
            return Err(UnitConversionError::SameUnit);
        }

        if numerator <= 0.0 {
            return Err(UnitConversionError::InvalidNumerator);
        }

        if denominator <= 0.0 {
            return Err(UnitConversionError::InvalidDenominator);
        }

        Ok(Self {
            from_unit,
            to_unit,
            numerator,
            denominator,
            ean_upc: None,
        })
    }

    /// 创建带 EAN/UPC 的单位换算
    pub fn with_ean_upc(mut self, ean_upc: impl Into<String>) -> Self {
        let ean = ean_upc.into();
        self.ean_upc = if ean.is_empty() { None } else { Some(ean) };
        self
    }

    /// 获取源单位
    pub fn source_unit(&self) -> &str {
        &self.from_unit
    }

    /// 获取目标单位
    pub fn target_unit(&self) -> &str {
        &self.to_unit
    }

    /// 获取分子
    pub fn numerator(&self) -> f64 {
        self.numerator
    }

    /// 获取分母
    pub fn denominator(&self) -> f64 {
        self.denominator
    }

    /// 获取 EAN/UPC 码
    pub fn ean_upc(&self) -> Option<&str> {
        self.ean_upc.as_deref()
    }

    /// 获取换算因子
    pub fn factor(&self) -> f64 {
        self.numerator / self.denominator
    }

    /// 执行换算
    pub fn convert(&self, quantity: f64) -> f64 {
        quantity * self.factor()
    }

    /// 获取反向换算
    pub fn reverse(&self) -> Self {
        Self {
            from_unit: self.to_unit.clone(),
            to_unit: self.from_unit.clone(),
            numerator: self.denominator,
            denominator: self.numerator,
            ean_upc: self.ean_upc.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_conversion() {
        let conv = UnitConversion::new("BOX", "PC", 12.0, 1.0).unwrap();
        assert_eq!(conv.source_unit(), "BOX");
        assert_eq!(conv.target_unit(), "PC");
        assert_eq!(conv.factor(), 12.0);
    }

    #[test]
    fn test_convert() {
        let conv = UnitConversion::new("BOX", "PC", 12.0, 1.0).unwrap();
        assert_eq!(conv.convert(5.0), 60.0);
    }

    #[test]
    fn test_reverse() {
        let conv = UnitConversion::new("BOX", "PC", 12.0, 1.0).unwrap();
        let reverse = conv.reverse();
        assert_eq!(reverse.source_unit(), "PC");
        assert_eq!(reverse.target_unit(), "BOX");
        assert!((reverse.factor() - 1.0 / 12.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_same_unit_error() {
        let result = UnitConversion::new("PC", "PC", 1.0, 1.0);
        assert!(matches!(result, Err(UnitConversionError::SameUnit)));
    }
}
