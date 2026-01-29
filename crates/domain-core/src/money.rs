//! 货币值对象

use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Sub};

/// 货币代码
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Currency(pub String);

impl Currency {
    pub fn new(code: &str) -> Self {
        Self(code.to_uppercase())
    }

    pub fn cny() -> Self {
        Self("CNY".to_string())
    }

    pub fn usd() -> Self {
        Self("USD".to_string())
    }

    pub fn eur() -> Self {
        Self("EUR".to_string())
    }
}

/// 金额值对象
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Money {
    /// 金额（以最小单位存储，如分）
    pub amount: i64,
    /// 货币代码
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: i64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn zero(currency: Currency) -> Self {
        Self {
            amount: 0,
            currency,
        }
    }

    pub fn cny(amount: i64) -> Self {
        Self::new(amount, Currency::cny())
    }

    pub fn usd(amount: i64) -> Self {
        Self::new(amount, Currency::usd())
    }

    /// 转换为浮点数（用于显示）
    pub fn to_decimal(&self) -> f64 {
        self.amount as f64 / 100.0
    }

    /// 从浮点数创建
    pub fn from_decimal(amount: f64, currency: Currency) -> Self {
        Self::new((amount * 100.0).round() as i64, currency)
    }

    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }

    pub fn is_positive(&self) -> bool {
        self.amount > 0
    }

    pub fn is_negative(&self) -> bool {
        self.amount < 0
    }

    pub fn abs(&self) -> Self {
        Self::new(self.amount.abs(), self.currency.clone())
    }

    pub fn negate(&self) -> Self {
        Self::new(-self.amount, self.currency.clone())
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        assert_eq!(
            self.currency, other.currency,
            "Cannot add money with different currencies"
        );
        Self::new(self.amount + other.amount, self.currency)
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        assert_eq!(
            self.currency, other.currency,
            "Cannot subtract money with different currencies"
        );
        Self::new(self.amount - other.amount, self.currency)
    }
}

impl Mul<i64> for Money {
    type Output = Self;

    fn mul(self, multiplier: i64) -> Self {
        Self::new(self.amount * multiplier, self.currency)
    }
}
