//! 仓储视图

use serde::{Deserialize, Serialize};

/// 物料仓储视图
///
/// 包含物料在特定工厂和存储位置的仓储数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageData {
    /// 工厂
    plant: String,
    /// 存储位置
    storage_location: String,

    // 仓储数据
    /// 仓库编号
    warehouse_number: String,
    /// 存储类型
    storage_type: String,
    /// 仓位
    storage_bin: String,
    /// 最大存储数量
    max_storage_quantity: f64,
    /// 最小存储数量
    min_storage_quantity: f64,
    /// 存储单位类型
    storage_unit_type: String,

    // 拣配
    /// 拣配区域
    picking_area: String,
    /// 存储区段
    storage_section: String,

    /// 删除标记
    deletion_flag: bool,
}

impl StorageData {
    /// 创建新的仓储视图
    pub fn new(plant: impl Into<String>, storage_location: impl Into<String>) -> Self {
        Self {
            plant: plant.into(),
            storage_location: storage_location.into(),
            warehouse_number: String::new(),
            storage_type: String::new(),
            storage_bin: String::new(),
            max_storage_quantity: 0.0,
            min_storage_quantity: 0.0,
            storage_unit_type: String::new(),
            picking_area: String::new(),
            storage_section: String::new(),
            deletion_flag: false,
        }
    }

    /// 获取工厂和存储位置的组合键
    pub fn key(&self) -> String {
        format!("{}_{}", self.plant, self.storage_location)
    }

    // Getters
    pub fn plant(&self) -> &str {
        &self.plant
    }

    pub fn storage_location(&self) -> &str {
        &self.storage_location
    }

    pub fn warehouse_number(&self) -> &str {
        &self.warehouse_number
    }

    pub fn storage_type(&self) -> &str {
        &self.storage_type
    }

    pub fn storage_bin(&self) -> &str {
        &self.storage_bin
    }

    pub fn max_storage_quantity(&self) -> f64 {
        self.max_storage_quantity
    }

    pub fn min_storage_quantity(&self) -> f64 {
        self.min_storage_quantity
    }

    pub fn storage_unit_type(&self) -> &str {
        &self.storage_unit_type
    }

    pub fn picking_area(&self) -> &str {
        &self.picking_area
    }

    pub fn storage_section(&self) -> &str {
        &self.storage_section
    }

    pub fn deletion_flag(&self) -> bool {
        self.deletion_flag
    }

    // Builder pattern setters
    pub fn with_warehouse_number(mut self, number: impl Into<String>) -> Self {
        self.warehouse_number = number.into();
        self
    }

    pub fn with_storage_type(mut self, storage_type: impl Into<String>) -> Self {
        self.storage_type = storage_type.into();
        self
    }

    pub fn with_storage_bin(mut self, bin: impl Into<String>) -> Self {
        self.storage_bin = bin.into();
        self
    }

    pub fn with_storage_quantities(mut self, min: f64, max: f64) -> Self {
        self.min_storage_quantity = min;
        self.max_storage_quantity = max;
        self
    }

    pub fn with_storage_unit_type(mut self, unit_type: impl Into<String>) -> Self {
        self.storage_unit_type = unit_type.into();
        self
    }

    pub fn with_picking_area(mut self, area: impl Into<String>) -> Self {
        self.picking_area = area.into();
        self
    }

    pub fn with_storage_section(mut self, section: impl Into<String>) -> Self {
        self.storage_section = section.into();
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
