//! 请求限流中间件
//!
//! 使用 Redis 实现分布式限流，防止暴力破解和 DDoS 攻击

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use std::sync::Arc;
use tracing::{debug, warn};

/// 限流配置
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Redis 连接 URL
    pub redis_url: String,
    /// 时间窗口（秒）
    pub window_secs: u64,
    /// 窗口内最大请求数
    pub max_requests: u64,
}

impl RateLimitConfig {
    pub fn new(redis_url: String, window_secs: u64, max_requests: u64) -> Self {
        Self {
            redis_url,
            window_secs,
            max_requests,
        }
    }
}

/// 限流中间件
pub async fn rate_limit_middleware(
    axum::extract::State(config): axum::extract::State<Arc<RateLimitConfig>>,
    req: Request,
    next: Next,
) -> Response {
    // 从请求中提取标识符（IP 地址 + 路径）
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let path = req.uri().path().to_string();
    let rate_limit_key = format!("rate_limit:{}:{}", client_ip, path);

    // 检查限流
    match check_rate_limit(&config, &rate_limit_key).await {
        Ok(allowed) => {
            if allowed {
                debug!(key = %rate_limit_key, "Request allowed");
                next.run(req).await
            } else {
                warn!(key = %rate_limit_key, "Rate limit exceeded");
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    "Too many requests. Please try again later.",
                )
                    .into_response()
            }
        }
        Err(e) => {
            // Redis 错误时，为了可用性，允许请求通过（但记录错误）
            warn!(error = %e, "Rate limit check failed, allowing request");
            next.run(req).await
        }
    }
}

/// 检查是否超过限流
async fn check_rate_limit(
    config: &RateLimitConfig,
    key: &str,
) -> Result<bool, String> {
    // 连接 Redis
    let client = redis::Client::open(config.redis_url.as_str())
        .map_err(|e| e.to_string())?;
    let mut con = client.get_multiplexed_async_connection().await
        .map_err(|e| e.to_string())?;

    // 使用 INCR + EXPIRE 实现简单的滑动窗口
    let count: u64 = con.incr(key, 1).await
        .map_err(|e| e.to_string())?;

    // 如果是第一次请求，设置过期时间
    if count == 1 {
        let _: () = con.expire(key, config.window_secs as i64).await
            .map_err(|e| e.to_string())?;
    }

    Ok(count <= config.max_requests)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::new("redis://localhost:6379".to_string(), 60, 10);
        assert_eq!(config.window_secs, 60);
        assert_eq!(config.max_requests, 10);
    }
}
