//! 会计视图

use serde::{Deserialize, Serialize};

use super::purchase_data::Money;
use crate::domain::enums::PriceControl;

/// 物料会计视图
///
/// 包含物料在特定工厂的评估和会计数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountingData {
    /// 工厂
    plant: String,
    /// 评估范围
    valuation_area: String,

    // 评估数据
    /// 评估类
    valuation_class: String,
    /// 价格控制 (S=标准价/V=移动平均价)
    price_control: PriceControl,
    /// 标准价格
    standard_price: Option<Money>,
    /// 移动平均价
    moving_average_price: Option<Money>,
    /// 价格单位
    price_unit: String,
    /// 价格单位数量
    price_unit_quantity: i32,

    // 总账科目
    /// 存货科目
    inventory_account: String,
    /// 价差科目
    price_difference_account: String,
    /// 成本要素
    cost_element: String,

    // 成本核算
    /// 成本核算批量
    costing_lot_size: String,
    /// 带数量结构
    with_qty_structure: bool,

    /// 删除标记
    deletion_flag: bool,
}

impl AccountingData {
    /// 创建新的会计视图
    pub fn new(plant: impl Into<String>, valuation_area: impl Into<String>) -> Self {
        Self {
            plant: plant.into(),
            valuation_area: valuation_area.into(),
            valuation_class: String::new(),
            price_control: PriceControl::default(),
            standard_price: None,
            moving_average_price: None,
            price_unit: String::new(),
            price_unit_quantity: 1,
            inventory_account: String::new(),
            price_difference_account: String::new(),
            cost_element: String::new(),
            costing_lot_size: String::new(),
            with_qty_structure: false,
            deletion_flag: false,
        }
    }

    /// 获取工厂和评估范围的组合键
    pub fn key(&self) -> String {
        format!("{}_{}", self.plant, self.valuation_area)
    }

    // Getters
    pub fn plant(&self) -> &str {
        &self.plant
    }

    pub fn valuation_area(&self) -> &str {
        &self.valuation_area
    }

    pub fn valuation_class(&self) -> &str {
        &self.valuation_class
    }

    pub fn price_control(&self) -> PriceControl {
        self.price_control
    }

    pub fn standard_price(&self) -> Option<&Money> {
        self.standard_price.as_ref()
    }

    pub fn moving_average_price(&self) -> Option<&Money> {
        self.moving_average_price.as_ref()
    }

    pub fn price_unit(&self) -> &str {
        &self.price_unit
    }

    pub fn price_unit_quantity(&self) -> i32 {
        self.price_unit_quantity
    }

    pub fn inventory_account(&self) -> &str {
        &self.inventory_account
    }

    pub fn price_difference_account(&self) -> &str {
        &self.price_difference_account
    }

    pub fn cost_element(&self) -> &str {
        &self.cost_element
    }

    pub fn costing_lot_size(&self) -> &str {
        &self.costing_lot_size
    }

    pub fn has_qty_structure(&self) -> bool {
        self.with_qty_structure
    }

    pub fn deletion_flag(&self) -> bool {
        self.deletion_flag
    }

    // Builder pattern setters
    pub fn with_valuation_class(mut self, class: impl Into<String>) -> Self {
        self.valuation_class = class.into();
        self
    }

    pub fn with_price_control(mut self, control: PriceControl) -> Self {
        self.price_control = control;
        self
    }

    pub fn with_standard_price(mut self, price: Money) -> Self {
        self.standard_price = Some(price);
        self
    }

    pub fn with_moving_average_price(mut self, price: Money) -> Self {
        self.moving_average_price = Some(price);
        self
    }

    pub fn with_price_unit(mut self, unit: impl Into<String>, quantity: i32) -> Self {
        self.price_unit = unit.into();
        self.price_unit_quantity = quantity;
        self
    }

    pub fn with_inventory_account(mut self, account: impl Into<String>) -> Self {
        self.inventory_account = account.into();
        self
    }

    pub fn with_price_difference_account(mut self, account: impl Into<String>) -> Self {
        self.price_difference_account = account.into();
        self
    }

    pub fn with_cost_element(mut self, element: impl Into<String>) -> Self {
        self.cost_element = element.into();
        self
    }

    pub fn with_costing_lot_size(mut self, size: impl Into<String>) -> Self {
        self.costing_lot_size = size.into();
        self
    }

    pub fn with_qty_structure(mut self, enabled: bool) -> Self {
        self.with_qty_structure = enabled;
        self
    }

    pub fn with_deletion_flag(mut self, flag: bool) -> Self {
        self.deletion_flag = flag;
        self
    }

    // Mutable setters
    pub fn set_deletion_flag(&mut self, flag: bool) {
        self.deletion_flag = flag;
    }
}
