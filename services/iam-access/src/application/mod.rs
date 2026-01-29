//! 应用层模块

pub mod authorization;
pub mod policy;
pub mod role;

pub use authorization::{
    AuthorizationCheckRequest, AuthorizationCheckResult, AuthorizationService, DecisionSource,
};
pub use policy::{
    CreatePolicyCommand, DeletePolicyCommand, PolicyCommandHandler, SetPolicyActiveCommand,
    UpdatePolicyCommand,
};
pub use role::{
    AssignPermissionsToRoleCommand, AssignRolesToUserCommand, CheckUserPermissionQuery,
    CreateRoleCommand, DeleteRoleCommand, GetRoleByCodeQuery, GetRoleQuery,
    GetUserPermissionsQuery, GetUserRolesQuery, ListRolesQuery, RemovePermissionsFromRoleCommand,
    RemoveRolesFromUserCommand, RoleCommandHandler, RoleListResult, RoleQueryHandler,
    SearchRolesQuery, SetRoleActiveCommand, UpdateRoleCommand,
};
