//! 共享领域服务

mod email_verification_service;
mod phone_verification_service;

pub use email_verification_service::*;
pub use phone_verification_service::*;
