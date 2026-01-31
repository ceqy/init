//! 重置密码命令

use cqrs_core::Command;
use serde::{Deserialize, Serialize};

/// 重置密码命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordCommand {
    /// 邮箱
    pub email: String,

    /// 重置令牌
    pub reset_token: String,

    /// 新密码
    pub new_password: String,

    /// 租户 ID
    pub tenant_id: String,
}

impl Command for ResetPasswordCommand {
    type Result = ();
}
