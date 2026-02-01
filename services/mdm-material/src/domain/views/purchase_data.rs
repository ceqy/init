//! 采购视图

use serde::{Deserialize, Serialize};

use crate::domain::enums::PurchaseMaterialStatus;

/// 货币金额
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Money {
    /// 货币代码 (如: CNY, USD)
    pub currency: String,
    /// 金额 (以最小货币单位表示)
    pub amount: i64,
    /// 小数位数
    pub decimal_places: i32,
}

impl Money {
    pub fn new(currency: impl Into<String>, amount: i64, decimal_places: i32) -> Self {
        Self {
            currency: currency.into(),
            amount,
            decimal_places,
        }
    }

    /// 获取实际金额值
    pub fn value(&self) -> f64 {
        self.amount as f64 / 10_f64.powi(self.decimal_places)
    }
}

/// 物料采购视图
///
/// 包含物料在特定采购组织的采购数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchaseData {
    /// 采购组织
    purchase_org: String,
    /// 工厂（可选）
    plant: String,

    // 采购数据
    /// 采购单位
    purchase_unit: String,
    /// 订单单位换算
    order_unit_conversion: f64,
    /// 采购组
    purchasing_group: String,
    /// 计划交货天数
    planned_delivery_days: i32,
    /// 超交容差 (%)
    over_delivery_tolerance: f64,
    /// 欠交容差 (%)
    under_delivery_tolerance: f64,
    /// 允许无限超交
    unlimited_over_delivery: bool,

    /// 首选供应商 ID
    preferred_vendor_id: String,

    // 价格
    /// 标准价格
    standard_price: Option<Money>,
    /// 最近采购价
    last_purchase_price: Option<Money>,
    /// 最近采购日期
    last_purchase_date: Option<chrono::DateTime<chrono::Utc>>,

    // 采购控制
    /// 自动采购订单
    automatic_po: bool,
    /// 货源清单
    source_list: String,

    /// 采购状态
    status: PurchaseMaterialStatus,
    /// 删除标记
    deletion_flag: bool,
}

impl PurchaseData {
    /// 创建新的采购视图
    pub fn new(purchase_org: impl Into<String>) -> Self {
        Self {
            purchase_org: purchase_org.into(),
            plant: String::new(),
            purchase_unit: String::new(),
            order_unit_conversion: 1.0,
            purchasing_group: String::new(),
            planned_delivery_days: 0,
            over_delivery_tolerance: 0.0,
            under_delivery_tolerance: 0.0,
            unlimited_over_delivery: false,
            preferred_vendor_id: String::new(),
            standard_price: None,
            last_purchase_price: None,
            last_purchase_date: None,
            automatic_po: false,
            source_list: String::new(),
            status: PurchaseMaterialStatus::default(),
            deletion_flag: false,
        }
    }

    /// 获取采购组织和工厂的组合键
    pub fn key(&self) -> String {
        if self.plant.is_empty() {
            self.purchase_org.clone()
        } else {
            format!("{}_{}", self.purchase_org, self.plant)
        }
    }

    // Getters
    pub fn purchase_org(&self) -> &str {
        &self.purchase_org
    }

    pub fn plant(&self) -> &str {
        &self.plant
    }

    pub fn purchase_unit(&self) -> &str {
        &self.purchase_unit
    }

    pub fn order_unit_conversion(&self) -> f64 {
        self.order_unit_conversion
    }

    pub fn purchasing_group(&self) -> &str {
        &self.purchasing_group
    }

    pub fn planned_delivery_days(&self) -> i32 {
        self.planned_delivery_days
    }

    pub fn over_delivery_tolerance(&self) -> f64 {
        self.over_delivery_tolerance
    }

    pub fn under_delivery_tolerance(&self) -> f64 {
        self.under_delivery_tolerance
    }

    pub fn unlimited_over_delivery(&self) -> bool {
        self.unlimited_over_delivery
    }

    pub fn preferred_vendor_id(&self) -> &str {
        &self.preferred_vendor_id
    }

    pub fn standard_price(&self) -> Option<&Money> {
        self.standard_price.as_ref()
    }

    pub fn last_purchase_price(&self) -> Option<&Money> {
        self.last_purchase_price.as_ref()
    }

    pub fn last_purchase_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.last_purchase_date
    }

    pub fn automatic_po(&self) -> bool {
        self.automatic_po
    }

    pub fn source_list(&self) -> &str {
        &self.source_list
    }

    pub fn status(&self) -> PurchaseMaterialStatus {
        self.status
    }

    pub fn deletion_flag(&self) -> bool {
        self.deletion_flag
    }

    // Builder pattern setters
    pub fn with_plant(mut self, plant: impl Into<String>) -> Self {
        self.plant = plant.into();
        self
    }

    pub fn with_purchase_unit(mut self, unit: impl Into<String>) -> Self {
        self.purchase_unit = unit.into();
        self
    }

    pub fn with_order_unit_conversion(mut self, conversion: f64) -> Self {
        self.order_unit_conversion = conversion;
        self
    }

    pub fn with_purchasing_group(mut self, group: impl Into<String>) -> Self {
        self.purchasing_group = group.into();
        self
    }

    pub fn with_planned_delivery_days(mut self, days: i32) -> Self {
        self.planned_delivery_days = days;
        self
    }

    pub fn with_delivery_tolerances(mut self, over: f64, under: f64, unlimited: bool) -> Self {
        self.over_delivery_tolerance = over;
        self.under_delivery_tolerance = under;
        self.unlimited_over_delivery = unlimited;
        self
    }

    pub fn with_preferred_vendor_id(mut self, vendor_id: impl Into<String>) -> Self {
        self.preferred_vendor_id = vendor_id.into();
        self
    }

    pub fn with_standard_price(mut self, price: Money) -> Self {
        self.standard_price = Some(price);
        self
    }

    pub fn with_last_purchase_price(
        mut self,
        price: Money,
        date: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.last_purchase_price = Some(price);
        self.last_purchase_date = Some(date);
        self
    }

    pub fn with_automatic_po(mut self, enabled: bool) -> Self {
        self.automatic_po = enabled;
        self
    }

    pub fn with_source_list(mut self, source_list: impl Into<String>) -> Self {
        self.source_list = source_list.into();
        self
    }

    pub fn with_status(mut self, status: PurchaseMaterialStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_deletion_flag(mut self, flag: bool) -> Self {
        self.deletion_flag = flag;
        self
    }

    // Mutable setters
    pub fn set_status(&mut self, status: PurchaseMaterialStatus) {
        self.status = status;
    }

    pub fn set_deletion_flag(&mut self, flag: bool) {
        self.deletion_flag = flag;
    }
}
