//! 角色应用层模块

pub mod commands;
pub mod handlers;
pub mod queries;
pub mod query_handlers;

pub use commands::*;
pub use handlers::RoleCommandHandler;
pub use queries::*;
pub use query_handlers::{RoleListResult, RoleQueryHandler};
