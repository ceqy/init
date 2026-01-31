//! adapter-clickhouse - ClickHouse 适配器
//!
//! 提供完整的 ClickHouse 集成，包括：
//! - 连接池管理
//! - 重试机制
//! - 批量写入
//! - 健康检查
//! - CDC 同步
//! - 分析 Repository 实现

mod batch;
mod client;
mod config;
mod health;
pub mod models;
mod repository;
mod retry;
pub mod sync;

pub use batch::*;
pub use client::*;
pub use config::*;
pub use health::*;
pub use repository::*;
pub use retry::*;
