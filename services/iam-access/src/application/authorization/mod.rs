//! 授权应用层模块

pub mod service;

pub use service::{
    AuthorizationCheckRequest, AuthorizationCheckResult, AuthorizationService, DecisionSource,
};
