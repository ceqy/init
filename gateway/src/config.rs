//! Gateway 配置

use std::env;

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub iam_auth_addr: String,
}

impl GatewayConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("GATEWAY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("GATEWAY_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secret-key".to_string()),
            iam_auth_addr: env::var("IAM_AUTH_GRPC_ADDR")
                .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string()),
        }
    }
}
