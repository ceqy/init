//! 工厂视图

use serde::{Deserialize, Serialize};

use crate::domain::enums::{PlantMaterialStatus, ProcurementType};

/// 物料工厂视图
///
/// 包含物料在特定工厂的 MRP、采购、库存等数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlantData {
    /// 工厂代码
    plant: String,
    /// 工厂名称
    plant_name: String,

    // MRP 数据
    /// MRP 类型 (PD/VB/ND 等)
    mrp_type: String,
    /// MRP 控制者
    mrp_controller: String,
    /// 再订货点
    reorder_point: f64,
    /// 安全库存
    safety_stock: f64,
    /// 最小批量
    minimum_lot_size: f64,
    /// 最大批量
    maximum_lot_size: f64,
    /// 固定批量
    fixed_lot_size: f64,
    /// 舍入值
    rounding_value: f64,
    /// 计划交货天数
    planned_delivery_days: i32,
    /// 收货处理天数
    gr_processing_days: i32,

    // 采购数据
    /// 采购类型
    procurement_type: ProcurementType,
    /// 特殊采购类型
    special_procurement: String,
    /// 生产计划员
    production_scheduler: String,

    // 库存数据
    /// 默认存储位置
    storage_location: String,
    /// 默认仓位
    storage_bin: String,
    /// 批次管理标识
    batch_management: bool,
    /// 序列号管理
    serial_number_profile: bool,

    /// ABC 分类 (A/B/C)
    abc_indicator: String,

    /// 工厂级状态
    status: PlantMaterialStatus,
    /// 删除标记
    deletion_flag: bool,
}

impl PlantData {
    /// 创建新的工厂视图
    pub fn new(plant: impl Into<String>) -> Self {
        Self {
            plant: plant.into(),
            plant_name: String::new(),
            mrp_type: String::new(),
            mrp_controller: String::new(),
            reorder_point: 0.0,
            safety_stock: 0.0,
            minimum_lot_size: 0.0,
            maximum_lot_size: 0.0,
            fixed_lot_size: 0.0,
            rounding_value: 0.0,
            planned_delivery_days: 0,
            gr_processing_days: 0,
            procurement_type: ProcurementType::default(),
            special_procurement: String::new(),
            production_scheduler: String::new(),
            storage_location: String::new(),
            storage_bin: String::new(),
            batch_management: false,
            serial_number_profile: false,
            abc_indicator: String::new(),
            status: PlantMaterialStatus::default(),
            deletion_flag: false,
        }
    }

    // Getters
    pub fn plant(&self) -> &str {
        &self.plant
    }

    pub fn plant_name(&self) -> &str {
        &self.plant_name
    }

    pub fn mrp_type(&self) -> &str {
        &self.mrp_type
    }

    pub fn mrp_controller(&self) -> &str {
        &self.mrp_controller
    }

    pub fn reorder_point(&self) -> f64 {
        self.reorder_point
    }

    pub fn safety_stock(&self) -> f64 {
        self.safety_stock
    }

    pub fn minimum_lot_size(&self) -> f64 {
        self.minimum_lot_size
    }

    pub fn maximum_lot_size(&self) -> f64 {
        self.maximum_lot_size
    }

    pub fn fixed_lot_size(&self) -> f64 {
        self.fixed_lot_size
    }

    pub fn rounding_value(&self) -> f64 {
        self.rounding_value
    }

    pub fn planned_delivery_days(&self) -> i32 {
        self.planned_delivery_days
    }

    pub fn gr_processing_days(&self) -> i32 {
        self.gr_processing_days
    }

    pub fn procurement_type(&self) -> ProcurementType {
        self.procurement_type
    }

    pub fn special_procurement(&self) -> &str {
        &self.special_procurement
    }

    pub fn production_scheduler(&self) -> &str {
        &self.production_scheduler
    }

    pub fn storage_location(&self) -> &str {
        &self.storage_location
    }

    pub fn storage_bin(&self) -> &str {
        &self.storage_bin
    }

    pub fn batch_management(&self) -> bool {
        self.batch_management
    }

    pub fn serial_number_profile(&self) -> bool {
        self.serial_number_profile
    }

    pub fn abc_indicator(&self) -> &str {
        &self.abc_indicator
    }

    pub fn status(&self) -> PlantMaterialStatus {
        self.status
    }

    pub fn deletion_flag(&self) -> bool {
        self.deletion_flag
    }

    // Setters (Builder pattern)
    pub fn with_plant_name(mut self, name: impl Into<String>) -> Self {
        self.plant_name = name.into();
        self
    }

    pub fn with_mrp_type(mut self, mrp_type: impl Into<String>) -> Self {
        self.mrp_type = mrp_type.into();
        self
    }

    pub fn with_mrp_controller(mut self, controller: impl Into<String>) -> Self {
        self.mrp_controller = controller.into();
        self
    }

    pub fn with_reorder_point(mut self, value: f64) -> Self {
        self.reorder_point = value;
        self
    }

    pub fn with_safety_stock(mut self, value: f64) -> Self {
        self.safety_stock = value;
        self
    }

    pub fn with_lot_sizes(mut self, min: f64, max: f64, fixed: f64) -> Self {
        self.minimum_lot_size = min;
        self.maximum_lot_size = max;
        self.fixed_lot_size = fixed;
        self
    }

    pub fn with_rounding_value(mut self, value: f64) -> Self {
        self.rounding_value = value;
        self
    }

    pub fn with_delivery_days(mut self, planned: i32, gr_processing: i32) -> Self {
        self.planned_delivery_days = planned;
        self.gr_processing_days = gr_processing;
        self
    }

    pub fn with_procurement_type(mut self, procurement_type: ProcurementType) -> Self {
        self.procurement_type = procurement_type;
        self
    }

    pub fn with_special_procurement(mut self, value: impl Into<String>) -> Self {
        self.special_procurement = value.into();
        self
    }

    pub fn with_production_scheduler(mut self, scheduler: impl Into<String>) -> Self {
        self.production_scheduler = scheduler.into();
        self
    }

    pub fn with_storage_location(mut self, location: impl Into<String>) -> Self {
        self.storage_location = location.into();
        self
    }

    pub fn with_storage_bin(mut self, bin: impl Into<String>) -> Self {
        self.storage_bin = bin.into();
        self
    }

    pub fn with_batch_management(mut self, enabled: bool) -> Self {
        self.batch_management = enabled;
        self
    }

    pub fn with_serial_number_profile(mut self, enabled: bool) -> Self {
        self.serial_number_profile = enabled;
        self
    }

    pub fn with_abc_indicator(mut self, indicator: impl Into<String>) -> Self {
        self.abc_indicator = indicator.into();
        self
    }

    pub fn with_status(mut self, status: PlantMaterialStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_deletion_flag(mut self, flag: bool) -> Self {
        self.deletion_flag = flag;
        self
    }

    // Mutable setters
    pub fn set_status(&mut self, status: PlantMaterialStatus) {
        self.status = status;
    }

    pub fn set_deletion_flag(&mut self, flag: bool) {
        self.deletion_flag = flag;
    }
}
