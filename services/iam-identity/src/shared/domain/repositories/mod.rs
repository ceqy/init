//! 共享仓储接口

mod tenant_repository;
mod user_repository;

pub use tenant_repository::*;
pub use user_repository::*;
