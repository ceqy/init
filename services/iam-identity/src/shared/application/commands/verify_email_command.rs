//! 验证邮箱命令

use cuba_cqrs_core::Command;
use serde::{Deserialize, Serialize};

/// 验证邮箱命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyEmailCommand {
    /// 用户 ID
    pub user_id: String,
    /// 验证码
    pub code: String,
    /// 租户 ID
    pub tenant_id: String,
}

impl Command for VerifyEmailCommand {
    type Result = VerifyEmailResult;
}

/// 验证邮箱结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyEmailResult {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
}
