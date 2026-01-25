//! UserService gRPC 实现
//!
//! 提供用户管理相关的 gRPC 接口

use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::shared::domain::repositories::UserRepository;

/// UserService 实现（占位符）
///
/// 注意：完整的 UserService 需要：
/// 1. 创建 proto/iam/user.proto
/// 2. 更新 build.rs 编译新 proto
/// 3. 实现完整的用户管理方法
pub struct UserServiceImpl {
    user_repo: Arc<dyn UserRepository>,
}

impl UserServiceImpl {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }
}

// TODO: 实现 UserService trait
// 当前 GetCurrentUser 和 UpdateProfile 仍在 AuthService 中
// 需要：
// 1. 创建 proto/iam/user.proto 定义 UserService
// 2. 从 auth.proto 中移除 GetCurrentUser 和 UpdateProfile
// 3. 在这里实现这些方法
