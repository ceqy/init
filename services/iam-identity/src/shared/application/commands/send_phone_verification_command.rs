//! 发送手机验证码命令

use cuba_cqrs_core::Command;
use serde::{Deserialize, Serialize};

/// 发送手机验证码命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPhoneVerificationCommand {
    /// 用户 ID
    pub user_id: String,
    /// 租户 ID
    pub tenant_id: String,
}

impl Command for SendPhoneVerificationCommand {
    type Result = SendPhoneVerificationResult;
}

/// 发送手机验证码结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPhoneVerificationResult {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 验证码有效期（秒）
    pub expires_in_seconds: i64,
}
