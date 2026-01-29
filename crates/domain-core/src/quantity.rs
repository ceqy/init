//! 数量值对象

use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Sub};

/// 计量单位
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Unit(pub String);

impl Unit {
    pub fn new(code: &str) -> Self {
        Self(code.to_string())
    }

    pub fn piece() -> Self {
        Self("PCS".to_string())
    }

    pub fn kilogram() -> Self {
        Self("KG".to_string())
    }

    pub fn meter() -> Self {
        Self("M".to_string())
    }

    pub fn liter() -> Self {
        Self("L".to_string())
    }
}

/// 数量值对象
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    /// 数量值（使用整数存储，乘以精度因子）
    pub value: i64,
    /// 精度因子（如 1000 表示 3 位小数）
    pub precision: i32,
    /// 计量单位
    pub unit: Unit,
}

impl Quantity {
    pub fn new(value: i64, precision: i32, unit: Unit) -> Self {
        Self {
            value,
            precision,
            unit,
        }
    }

    pub fn zero(unit: Unit) -> Self {
        Self::new(0, 1000, unit)
    }

    pub fn from_decimal(value: f64, unit: Unit) -> Self {
        Self::new((value * 1000.0).round() as i64, 1000, unit)
    }

    pub fn to_decimal(&self) -> f64 {
        self.value as f64 / self.precision as f64
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    pub fn is_positive(&self) -> bool {
        self.value > 0
    }

    pub fn is_negative(&self) -> bool {
        self.value < 0
    }
}

impl Add for Quantity {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        assert_eq!(
            self.unit, other.unit,
            "Cannot add quantities with different units"
        );
        assert_eq!(
            self.precision, other.precision,
            "Cannot add quantities with different precisions"
        );
        Self::new(self.value + other.value, self.precision, self.unit)
    }
}

impl Sub for Quantity {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        assert_eq!(
            self.unit, other.unit,
            "Cannot subtract quantities with different units"
        );
        assert_eq!(
            self.precision, other.precision,
            "Cannot subtract quantities with different precisions"
        );
        Self::new(self.value - other.value, self.precision, self.unit)
    }
}

impl Mul<i64> for Quantity {
    type Output = Self;

    fn mul(self, multiplier: i64) -> Self {
        Self::new(self.value * multiplier, self.precision, self.unit)
    }
}
