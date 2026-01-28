//! 基础设施层

pub mod persistence;

pub use persistence::{
    PostgresPermissionRepository,
    PostgresPolicyRepository,
    PostgresRoleRepository,
    PostgresRolePermissionRepository,
    PostgresUserRoleRepository,
};
