//! 视图模块

mod accounting_data;
mod plant_data;
mod purchase_data;
mod quality_data;
mod sales_data;
mod storage_data;

pub use accounting_data::AccountingData;
pub use plant_data::PlantData;
pub use purchase_data::{Money, PurchaseData};
pub use quality_data::QualityData;
pub use sales_data::SalesData;
pub use storage_data::StorageData;
