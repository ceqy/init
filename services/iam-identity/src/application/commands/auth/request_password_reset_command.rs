//! 请求密码重置命令

use cqrs_core::Command;
use serde::{Deserialize, Serialize};

/// 请求密码重置命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPasswordResetCommand {
    /// 邮箱
    pub email: String,

    /// 租户 ID
    pub tenant_id: String,

    /// 重置链接基础 URL（如 https://app.example.com/reset-password）
    pub reset_url_base: String,
}

impl Command for RequestPasswordResetCommand {
    type Result = ();
}
