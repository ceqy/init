//! adapter-kafka - Kafka 适配器
//!
//! 提供完整的 Kafka 功能支持：
//! - 消息生产（单条、批量、带 key）
//! - 消息消费（重试、DLQ）
//! - Topic 管理（创建、删除、分区）
//! - 健康检查
//! - 完整配置（SASL、SSL、压缩）

mod admin;
mod batch;
mod config;
mod consumer;
mod health;
mod producer;

pub use admin::*;
pub use batch::*;
pub use config::*;
pub use consumer::*;
pub use health::*;
pub use producer::*;
