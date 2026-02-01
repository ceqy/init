//! 物料状态枚举（工厂/销售/采购级别）

use serde::{Deserialize, Serialize};

/// 工厂级物料状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PlantMaterialStatus {
    /// 未指定
    #[default]
    Unspecified,
    /// 活跃
    Active,
    /// 采购冻结
    BlockedProcurement,
    /// 生产冻结
    BlockedProduction,
    /// 全部冻结
    BlockedAll,
}

impl PlantMaterialStatus {
    /// 是否允许采购
    pub fn allows_procurement(&self) -> bool {
        matches!(self, PlantMaterialStatus::Active | PlantMaterialStatus::BlockedProduction)
    }

    /// 是否允许生产
    pub fn allows_production(&self) -> bool {
        matches!(self, PlantMaterialStatus::Active | PlantMaterialStatus::BlockedProcurement)
    }

    /// 是否完全冻结
    pub fn is_fully_blocked(&self) -> bool {
        matches!(self, PlantMaterialStatus::BlockedAll)
    }
}

impl From<i32> for PlantMaterialStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => PlantMaterialStatus::Active,
            2 => PlantMaterialStatus::BlockedProcurement,
            3 => PlantMaterialStatus::BlockedProduction,
            4 => PlantMaterialStatus::BlockedAll,
            _ => PlantMaterialStatus::Unspecified,
        }
    }
}

impl From<PlantMaterialStatus> for i32 {
    fn from(status: PlantMaterialStatus) -> Self {
        match status {
            PlantMaterialStatus::Unspecified => 0,
            PlantMaterialStatus::Active => 1,
            PlantMaterialStatus::BlockedProcurement => 2,
            PlantMaterialStatus::BlockedProduction => 3,
            PlantMaterialStatus::BlockedAll => 4,
        }
    }
}

/// 销售级物料状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SalesMaterialStatus {
    /// 未指定
    #[default]
    Unspecified,
    /// 活跃
    Active,
    /// 冻结
    Blocked,
}

impl SalesMaterialStatus {
    /// 是否允许销售
    pub fn allows_sales(&self) -> bool {
        matches!(self, SalesMaterialStatus::Active)
    }
}

impl From<i32> for SalesMaterialStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => SalesMaterialStatus::Active,
            2 => SalesMaterialStatus::Blocked,
            _ => SalesMaterialStatus::Unspecified,
        }
    }
}

impl From<SalesMaterialStatus> for i32 {
    fn from(status: SalesMaterialStatus) -> Self {
        match status {
            SalesMaterialStatus::Unspecified => 0,
            SalesMaterialStatus::Active => 1,
            SalesMaterialStatus::Blocked => 2,
        }
    }
}

/// 采购级物料状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PurchaseMaterialStatus {
    /// 未指定
    #[default]
    Unspecified,
    /// 活跃
    Active,
    /// 冻结
    Blocked,
}

impl PurchaseMaterialStatus {
    /// 是否允许采购
    pub fn allows_purchase(&self) -> bool {
        matches!(self, PurchaseMaterialStatus::Active)
    }
}

impl From<i32> for PurchaseMaterialStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => PurchaseMaterialStatus::Active,
            2 => PurchaseMaterialStatus::Blocked,
            _ => PurchaseMaterialStatus::Unspecified,
        }
    }
}

impl From<PurchaseMaterialStatus> for i32 {
    fn from(status: PurchaseMaterialStatus) -> Self {
        match status {
            PurchaseMaterialStatus::Unspecified => 0,
            PurchaseMaterialStatus::Active => 1,
            PurchaseMaterialStatus::Blocked => 2,
        }
    }
}
