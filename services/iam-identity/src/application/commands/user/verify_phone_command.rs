//! 验证手机命令

use cqrs_core::Command;
use serde::{Deserialize, Serialize};

/// 验证手机命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPhoneCommand {
    /// 用户 ID
    pub user_id: String,
    /// 验证码
    pub code: String,
    /// 租户 ID
    pub tenant_id: String,
}

impl Command for VerifyPhoneCommand {
    type Result = VerifyPhoneResult;
}

/// 验证手机结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPhoneResult {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
}
