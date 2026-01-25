//! 共享仓储接口

mod email_verification_repository;
mod phone_verification_repository;
mod tenant_repository;
mod user_repository;

pub use email_verification_repository::*;
pub use phone_verification_repository::*;
pub use tenant_repository::*;
pub use user_repository::*;
