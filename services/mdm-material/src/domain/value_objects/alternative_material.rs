//! 替代物料值对象

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::MaterialId;

/// 替代物料关系
///
/// 表示一个物料可以被另一个物料替代的关系
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlternativeMaterial {
    /// 替代物料 ID
    material_id: MaterialId,
    /// 替代物料编号
    material_number: String,
    /// 替代物料描述
    description: String,
    /// 适用工厂（可选，为空表示所有工厂）
    plant: Option<String>,
    /// 优先级（数字越小优先级越高）
    priority: i32,
    /// 生效日期
    valid_from: Option<DateTime<Utc>>,
    /// 失效日期
    valid_to: Option<DateTime<Utc>>,
}

impl AlternativeMaterial {
    /// 创建新的替代物料关系
    pub fn new(
        material_id: MaterialId,
        material_number: impl Into<String>,
        description: impl Into<String>,
        priority: i32,
    ) -> Self {
        Self {
            material_id,
            material_number: material_number.into(),
            description: description.into(),
            plant: None,
            priority,
            valid_from: None,
            valid_to: None,
        }
    }

    /// 设置适用工厂
    pub fn with_plant(mut self, plant: impl Into<String>) -> Self {
        let p = plant.into();
        self.plant = if p.is_empty() { None } else { Some(p) };
        self
    }

    /// 设置有效期
    pub fn with_validity(
        mut self,
        valid_from: Option<DateTime<Utc>>,
        valid_to: Option<DateTime<Utc>>,
    ) -> Self {
        self.valid_from = valid_from;
        self.valid_to = valid_to;
        self
    }

    /// 获取替代物料 ID
    pub fn material_id(&self) -> &MaterialId {
        &self.material_id
    }

    /// 获取替代物料编号
    pub fn material_number(&self) -> &str {
        &self.material_number
    }

    /// 获取替代物料描述
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 获取适用工厂
    pub fn plant(&self) -> Option<&str> {
        self.plant.as_deref()
    }

    /// 获取优先级
    pub fn priority(&self) -> i32 {
        self.priority
    }

    /// 获取生效日期
    pub fn valid_from(&self) -> Option<DateTime<Utc>> {
        self.valid_from
    }

    /// 获取失效日期
    pub fn valid_to(&self) -> Option<DateTime<Utc>> {
        self.valid_to
    }

    /// 检查在指定日期是否有效
    pub fn is_valid_at(&self, date: DateTime<Utc>) -> bool {
        let after_start = self.valid_from.is_none_or(|from| date >= from);
        let before_end = self.valid_to.is_none_or(|to| date <= to);
        after_start && before_end
    }

    /// 检查当前是否有效
    pub fn is_currently_valid(&self) -> bool {
        self.is_valid_at(Utc::now())
    }

    /// 检查是否适用于指定工厂
    pub fn is_applicable_to_plant(&self, plant: &str) -> bool {
        self.plant.as_ref().is_none_or(|p| p == plant)
    }
}
