//! domain-core - 跨 context 的领域核心类型
//!
//! 包含极少数需要跨 bounded context 共享的值对象

mod entity;
mod money;
mod quantity;

pub use entity::*;
pub use money::*;
pub use quantity::*;

// Re-export common types
pub use common::{AuditInfo, TenantId, UserId};
