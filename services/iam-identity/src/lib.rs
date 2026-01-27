//! IAM Identity Service Library
//!
//! 模块化架构：
//! - `shared`: 共享层（User 实体、值对象、仓储）
//! - `auth`: 认证模块（登录、令牌、2FA、会话）
//! - `user`: 用户模块（注册、CRUD、验证、社交绑定）
//! - `oauth`: OAuth2/OIDC 模块

pub mod auth;
pub mod config;
pub mod error;
pub mod oauth;
pub mod shared;
pub mod user;
