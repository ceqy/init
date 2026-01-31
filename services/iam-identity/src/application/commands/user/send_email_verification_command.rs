//! 发送邮箱验证码命令

use cqrs_core::Command;
use serde::{Deserialize, Serialize};

/// 发送邮箱验证码命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailVerificationCommand {
    /// 用户 ID
    pub user_id: String,
    /// 租户 ID
    pub tenant_id: String,
}

impl Command for SendEmailVerificationCommand {
    type Result = SendEmailVerificationResult;
}

/// 发送邮箱验证码结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailVerificationResult {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 验证码有效期（秒）
    pub expires_in_seconds: i64,
}
