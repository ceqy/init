//! 应用层模块

pub mod authorization;
pub mod policy;
pub mod role;

pub use authorization::{
    AuthorizationCheckRequest,
    AuthorizationCheckResult,
    AuthorizationService,
    DecisionSource,
};
pub use policy::{PolicyCommandHandler, CreatePolicyCommand, UpdatePolicyCommand, DeletePolicyCommand, SetPolicyActiveCommand};
pub use role::{
    RoleCommandHandler, 
    RoleQueryHandler, 
    RoleListResult,
    CreateRoleCommand, 
    UpdateRoleCommand, 
    DeleteRoleCommand, 
    SetRoleActiveCommand,
    AssignPermissionsToRoleCommand,
    RemovePermissionsFromRoleCommand,
    AssignRolesToUserCommand,
    RemoveRolesFromUserCommand,
    GetRoleQuery,
    GetRoleByCodeQuery,
    ListRolesQuery,
    SearchRolesQuery,
    GetUserRolesQuery,
    GetUserPermissionsQuery,
    CheckUserPermissionQuery,
};
