//! 质量视图

use serde::{Deserialize, Serialize};

/// 物料质量视图
///
/// 包含物料在特定工厂的质量管理数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityData {
    /// 工厂
    plant: String,

    // 质量数据
    /// 启用检验
    inspection_active: bool,
    /// 检验类型
    inspection_type: String,
    /// 检验计划
    inspection_plan: String,
    /// 抽样比例
    sample_percentage: f64,
    /// 需要证书
    certificate_required: bool,
    /// 质量管理控制键
    quality_management_control_key: String,

    // 保质期
    /// 保质期天数
    shelf_life_days: i32,
    /// 剩余保质期要求
    remaining_shelf_life: i32,
    /// 总保质期
    total_shelf_life: i32,

    /// 删除标记
    deletion_flag: bool,
}

impl QualityData {
    /// 创建新的质量视图
    pub fn new(plant: impl Into<String>) -> Self {
        Self {
            plant: plant.into(),
            inspection_active: false,
            inspection_type: String::new(),
            inspection_plan: String::new(),
            sample_percentage: 0.0,
            certificate_required: false,
            quality_management_control_key: String::new(),
            shelf_life_days: 0,
            remaining_shelf_life: 0,
            total_shelf_life: 0,
            deletion_flag: false,
        }
    }

    // Getters
    pub fn plant(&self) -> &str {
        &self.plant
    }

    pub fn inspection_active(&self) -> bool {
        self.inspection_active
    }

    pub fn inspection_type(&self) -> &str {
        &self.inspection_type
    }

    pub fn inspection_plan(&self) -> &str {
        &self.inspection_plan
    }

    pub fn sample_percentage(&self) -> f64 {
        self.sample_percentage
    }

    pub fn certificate_required(&self) -> bool {
        self.certificate_required
    }

    pub fn quality_management_control_key(&self) -> &str {
        &self.quality_management_control_key
    }

    pub fn shelf_life_days(&self) -> i32 {
        self.shelf_life_days
    }

    pub fn remaining_shelf_life(&self) -> i32 {
        self.remaining_shelf_life
    }

    pub fn total_shelf_life(&self) -> i32 {
        self.total_shelf_life
    }

    pub fn deletion_flag(&self) -> bool {
        self.deletion_flag
    }

    // Builder pattern setters
    pub fn with_inspection_active(mut self, active: bool) -> Self {
        self.inspection_active = active;
        self
    }

    pub fn with_inspection_type(mut self, inspection_type: impl Into<String>) -> Self {
        self.inspection_type = inspection_type.into();
        self
    }

    pub fn with_inspection_plan(mut self, plan: impl Into<String>) -> Self {
        self.inspection_plan = plan.into();
        self
    }

    pub fn with_sample_percentage(mut self, percentage: f64) -> Self {
        self.sample_percentage = percentage;
        self
    }

    pub fn with_certificate_required(mut self, required: bool) -> Self {
        self.certificate_required = required;
        self
    }

    pub fn with_quality_management_control_key(mut self, key: impl Into<String>) -> Self {
        self.quality_management_control_key = key.into();
        self
    }

    pub fn with_shelf_life(mut self, days: i32, remaining: i32, total: i32) -> Self {
        self.shelf_life_days = days;
        self.remaining_shelf_life = remaining;
        self.total_shelf_life = total;
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
