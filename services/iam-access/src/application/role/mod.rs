//! 角色应用层模块

pub mod commands;
pub mod queries;
pub mod handlers;
pub mod query_handlers;

pub use commands::*;
pub use queries::*;
pub use handlers::RoleCommandHandler;
pub use query_handlers::{RoleQueryHandler, RoleListResult};
