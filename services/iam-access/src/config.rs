//! 服务配置

use serde::Deserialize;

/// IAM Access 服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct AccessServiceConfig {
    pub database_url: String,
    pub redis_url: String,
}
