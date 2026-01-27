#![allow(dead_code)]
#![allow(unused_imports)]

//! IAM Identity Service Library
//!
//! 统一的单体模块化架构：
//! - `domain`: 领域层（实体、值对象、仓储接口、领域服务、事件）
//! - `application`: 应用层（命令、查询、处理器、DTO）
//! - `infrastructure`: 基础设施层（持久化、缓存、外部服务）
//! - `api`: API 层（gRPC 服务）

pub mod api;
pub mod application;
pub mod config;
pub mod domain;
pub mod error;
pub mod infrastructure;
