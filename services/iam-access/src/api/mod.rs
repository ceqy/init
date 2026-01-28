//! API 层模块

pub mod grpc;
pub mod proto;

pub use grpc::{RbacServiceImpl, AuthorizationServiceImpl};
pub use proto::{rbac, policy, authorization};
