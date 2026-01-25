//! 服务配置

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AuthServiceConfig {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: u64,
    pub refresh_token_expires_in: u64,
}
