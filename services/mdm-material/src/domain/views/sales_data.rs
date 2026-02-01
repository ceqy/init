//! 销售视图

use serde::{Deserialize, Serialize};

use crate::domain::enums::SalesMaterialStatus;

/// 物料销售视图
///
/// 包含物料在特定销售组织和分销渠道的销售数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalesData {
    /// 销售组织
    sales_org: String,
    /// 分销渠道
    distribution_channel: String,
    /// 产品组
    division: String,

    // 销售数据
    /// 销售单位
    sales_unit: String,
    /// 最小订单数量
    minimum_order_quantity: f64,
    /// 最小交货数量
    minimum_delivery_quantity: f64,
    /// 交货单位
    delivery_unit: String,
    /// 交货天数
    delivery_days: i32,

    // 定价
    /// 定价参考物料
    pricing_reference_material: String,
    /// 物料定价组
    material_pricing_group: String,
    /// 科目分配组
    account_assignment_group: String,

    /// 税务分类
    tax_classification: String,

    /// 可用性检查组
    availability_check: String,

    /// 销售状态
    status: SalesMaterialStatus,
    /// 删除标记
    deletion_flag: bool,
}

impl SalesData {
    /// 创建新的销售视图
    pub fn new(
        sales_org: impl Into<String>,
        distribution_channel: impl Into<String>,
    ) -> Self {
        Self {
            sales_org: sales_org.into(),
            distribution_channel: distribution_channel.into(),
            division: String::new(),
            sales_unit: String::new(),
            minimum_order_quantity: 0.0,
            minimum_delivery_quantity: 0.0,
            delivery_unit: String::new(),
            delivery_days: 0,
            pricing_reference_material: String::new(),
            material_pricing_group: String::new(),
            account_assignment_group: String::new(),
            tax_classification: String::new(),
            availability_check: String::new(),
            status: SalesMaterialStatus::default(),
            deletion_flag: false,
        }
    }

    /// 获取销售组织和分销渠道的组合键
    pub fn key(&self) -> String {
        format!("{}_{}", self.sales_org, self.distribution_channel)
    }

    // Getters
    pub fn sales_org(&self) -> &str {
        &self.sales_org
    }

    pub fn distribution_channel(&self) -> &str {
        &self.distribution_channel
    }

    pub fn division(&self) -> &str {
        &self.division
    }

    pub fn sales_unit(&self) -> &str {
        &self.sales_unit
    }

    pub fn minimum_order_quantity(&self) -> f64 {
        self.minimum_order_quantity
    }

    pub fn minimum_delivery_quantity(&self) -> f64 {
        self.minimum_delivery_quantity
    }

    pub fn delivery_unit(&self) -> &str {
        &self.delivery_unit
    }

    pub fn delivery_days(&self) -> i32 {
        self.delivery_days
    }

    pub fn pricing_reference_material(&self) -> &str {
        &self.pricing_reference_material
    }

    pub fn material_pricing_group(&self) -> &str {
        &self.material_pricing_group
    }

    pub fn account_assignment_group(&self) -> &str {
        &self.account_assignment_group
    }

    pub fn tax_classification(&self) -> &str {
        &self.tax_classification
    }

    pub fn availability_check(&self) -> &str {
        &self.availability_check
    }

    pub fn status(&self) -> SalesMaterialStatus {
        self.status
    }

    pub fn deletion_flag(&self) -> bool {
        self.deletion_flag
    }

    // Builder pattern setters
    pub fn with_division(mut self, division: impl Into<String>) -> Self {
        self.division = division.into();
        self
    }

    pub fn with_sales_unit(mut self, unit: impl Into<String>) -> Self {
        self.sales_unit = unit.into();
        self
    }

    pub fn with_minimum_order_quantity(mut self, quantity: f64) -> Self {
        self.minimum_order_quantity = quantity;
        self
    }

    pub fn with_minimum_delivery_quantity(mut self, quantity: f64) -> Self {
        self.minimum_delivery_quantity = quantity;
        self
    }

    pub fn with_delivery_unit(mut self, unit: impl Into<String>) -> Self {
        self.delivery_unit = unit.into();
        self
    }

    pub fn with_delivery_days(mut self, days: i32) -> Self {
        self.delivery_days = days;
        self
    }

    pub fn with_pricing_reference_material(mut self, material: impl Into<String>) -> Self {
        self.pricing_reference_material = material.into();
        self
    }

    pub fn with_material_pricing_group(mut self, group: impl Into<String>) -> Self {
        self.material_pricing_group = group.into();
        self
    }

    pub fn with_account_assignment_group(mut self, group: impl Into<String>) -> Self {
        self.account_assignment_group = group.into();
        self
    }

    pub fn with_tax_classification(mut self, classification: impl Into<String>) -> Self {
        self.tax_classification = classification.into();
        self
    }

    pub fn with_availability_check(mut self, check: impl Into<String>) -> Self {
        self.availability_check = check.into();
        self
    }

    pub fn with_status(mut self, status: SalesMaterialStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_deletion_flag(mut self, flag: bool) -> Self {
        self.deletion_flag = flag;
        self
    }

    // Mutable setters
    pub fn set_status(&mut self, status: SalesMaterialStatus) {
        self.status = status;
    }

    pub fn set_deletion_flag(&mut self, flag: bool) {
        self.deletion_flag = flag;
    }
}
