//! RBAC 角色权限领域模块

pub mod permission;
pub mod role;
pub mod repository;
pub mod events;

pub use permission::{Permission, PermissionId};
pub use role::{Role, RoleId};
pub use repository::{
    PermissionRepository,
    RolePermissionRepository,
    RoleRepository,
    UserRoleRepository,
};
