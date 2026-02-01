//! 值对象模块

mod alternative_material;
mod ids;
mod localized_text;
mod material_number;
mod unit_conversion;

pub use alternative_material::AlternativeMaterial;
pub use ids::{MaterialGroupId, MaterialId, MaterialTypeId};
pub use localized_text::LocalizedText;
pub use material_number::{MaterialNumber, MaterialNumberError};
pub use unit_conversion::{UnitConversion, UnitConversionError};
