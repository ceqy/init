//! 数据状态枚举

use serde::{Deserialize, Serialize};

/// 数据状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DataStatus {
    /// 草稿
    #[default]
    Draft,
    /// 活跃
    Active,
    /// 停用
    Inactive,
    /// 冻结
    Blocked,
    /// 标记删除
    MarkedForDeletion,
}

impl DataStatus {
    /// 是否可以激活
    pub fn can_activate(&self) -> bool {
        matches!(self, DataStatus::Draft | DataStatus::Inactive)
    }

    /// 是否可以停用
    pub fn can_deactivate(&self) -> bool {
        matches!(self, DataStatus::Active)
    }

    /// 是否可以冻结
    pub fn can_block(&self) -> bool {
        matches!(self, DataStatus::Active | DataStatus::Inactive)
    }

    /// 是否可以标记删除
    pub fn can_mark_for_deletion(&self) -> bool {
        !matches!(self, DataStatus::MarkedForDeletion)
    }

    /// 是否为活跃状态
    pub fn is_active(&self) -> bool {
        matches!(self, DataStatus::Active)
    }
}

impl From<i32> for DataStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => DataStatus::Draft,
            2 => DataStatus::Active,
            3 => DataStatus::Blocked,
            4 => DataStatus::Inactive,
            5 => DataStatus::MarkedForDeletion,
            _ => DataStatus::Draft,
        }
    }
}

impl From<DataStatus> for i32 {
    fn from(status: DataStatus) -> Self {
        match status {
            DataStatus::Draft => 1,
            DataStatus::Active => 2,
            DataStatus::Blocked => 3,
            DataStatus::Inactive => 4,
            DataStatus::MarkedForDeletion => 5,
        }
    }
}
