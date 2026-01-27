//! 用户仓储接口

pub mod email_verification_repository;
pub mod phone_verification_repository;
pub mod tenant_repository;
pub mod user_repository;

pub use email_verification_repository::*;
pub use phone_verification_repository::*;
pub use tenant_repository::*;
pub use user_repository::*;
