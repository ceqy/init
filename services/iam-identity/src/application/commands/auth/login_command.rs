//! 登录命令

use cuba_cqrs_core::Command;

use crate::application::dto::auth::TokenPair;

/// 登录命令
#[derive(Debug, Clone)]
pub struct LoginCommand {
    pub username: String,
    pub password: String,
    pub tenant_id: String,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
}

impl Command for LoginCommand {
    type Result = LoginResult;
}

/// 登录结果
#[derive(Debug, Clone)]
pub struct LoginResult {
    pub tokens: Option<TokenPair>,
    pub user_id: String,
    pub require_2fa: bool,
    pub session_id: Option<String>,
}
