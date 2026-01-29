//! RBAC 角色权限领域模块

#![allow(clippy::module_inception)]

pub mod events;
pub mod permission;
pub mod repository;
pub mod role;

pub use permission::{Permission, PermissionId};
pub use repository::{
    PermissionRepository, RolePermissionRepository, RoleRepository, UserRoleRepository,
};
pub use role::{Role, RoleId};
