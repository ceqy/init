//! Gateway 配置

use std::env;

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub iam_endpoint: String,
    pub redis_url: String,
    pub cors_allowed_origins: Vec<String>,
}

impl GatewayConfig {
    pub fn from_env() -> Self {
        // 安全关键配置必须从环境变量读取，不提供默认值
        let jwt_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET environment variable must be set. Generate a secure random key (at least 32 bytes).");

        // 验证 JWT 密钥强度
        if jwt_secret.len() < 32 {
            panic!("JWT_SECRET must be at least 32 characters long for security.");
        }

        // Redis URL 必须显式配置
        let redis_url = env::var("REDIS_URL")
            .expect("REDIS_URL environment variable must be set (e.g., redis://localhost:6379 or redis://:password@localhost:6379).");

        // CORS 配置：从环境变量读取允许的源，用逗号分隔
        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            host: env::var("GATEWAY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("GATEWAY_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            jwt_secret,
            iam_endpoint: env::var("IAM_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string()),
            redis_url,
            cors_allowed_origins,
        }
    }
}
