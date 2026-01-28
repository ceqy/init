//! gRPC 服务模块

pub mod rbac_service;
pub mod authorization_service;

pub use rbac_service::RbacServiceImpl;
pub use authorization_service::AuthorizationServiceImpl;
