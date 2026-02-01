//! 仓储接口模块

mod material_group_repository;
mod material_repository;
mod material_type_repository;

pub use material_group_repository::MaterialGroupRepository;
pub use material_repository::MaterialRepository;
pub use material_type_repository::MaterialTypeRepository;
