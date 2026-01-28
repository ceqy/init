//! 基础设施层

pub mod persistence;
pub mod events;
pub mod cache;

pub use persistence::{
    PostgresPermissionRepository,
    PostgresPolicyRepository,
    PostgresRoleRepository,
    PostgresRolePermissionRepository,
    PostgresUserRoleRepository,
};
