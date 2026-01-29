//! 基础设施层

pub mod cache;
pub mod events;
pub mod persistence;

pub use persistence::{
    PostgresPermissionRepository, PostgresPolicyRepository, PostgresRolePermissionRepository,
    PostgresRoleRepository, PostgresUserRoleRepository,
};
