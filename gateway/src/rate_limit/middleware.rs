//! 限流中间件
//!
//! 集成用户等级检测、接口分类和限流检查

use crate::middleware::AuthContext;
use crate::rate_limit::classifier::EndpointClassifier;
use crate::rate_limit::config::ConfigManager;
use crate::rate_limit::limiter::RateLimiter;
use crate::rate_limit::types::{RateLimitResult, UserTier};
use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::{debug, warn};

/// 限流中间件状态
#[derive(Clone)]
pub struct RateLimitMiddleware {
    /// 配置管理器
    pub config_manager: Arc<ConfigManager>,
    /// 限流器
    pub rate_limiter: Arc<RateLimiter>,
    /// 接口分类器
    pub classifier: Arc<EndpointClassifier>,
}

impl RateLimitMiddleware {
    /// 创建新的限流中间件
    pub fn new(
        config_manager: Arc<ConfigManager>,
        rate_limiter: Arc<RateLimiter>,
        classifier: Arc<EndpointClassifier>,
    ) -> Self {
        Self {
            config_manager,
            rate_limiter,
            classifier,
        }
    }

    /// 提取客户端标识符 (Sync)
    fn extract_identifier(&self, req: &Request) -> String {
        // 检查是否已认证
        if let Some(ctx) = req.extensions().get::<AuthContext>() {
            return format!("user:{}", ctx.claims.sub);
        }

        // 未认证，使用 IP 地址
        let ip = req
            .headers()
            .get(header::FORWARDED)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(|s| s.trim())
            .or_else(|| {
                req.headers()
                    .get("x-forwarded-for")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.split(',').next())
                    .map(|s| s.trim())
            })
            .or_else(|| req.headers().get("x-real-ip").and_then(|v| v.to_str().ok()))
            .unwrap_or("unknown");

        format!("ip:{}", ip)
    }

    /// 检测用户等级 (Sync)
    fn detect_user_tier(&self, req: &Request) -> UserTier {
        UserTier::from_auth_context(req.extensions().get::<AuthContext>())
    }

    /// 为响应添加限流相关头
    fn add_rate_limit_headers(response: &mut Response, result: &RateLimitResult) {
        let headers = response.headers_mut();

        // X-RateLimit-Limit
        if let Ok(val) = HeaderValue::from_str(&result.limit.to_string()) {
            headers.insert("X-RateLimit-Limit", val);
        }

        // X-RateLimit-Remaining
        if let Ok(val) = HeaderValue::from_str(&result.remaining.to_string()) {
            headers.insert("X-RateLimit-Remaining", val);
        }

        // X-RateLimit-Reset
        if let Ok(val) = HeaderValue::from_str(&result.reset_at.to_string()) {
            headers.insert("X-RateLimit-Reset", val);
        }

        // Retry-After
        if let Some(val) = result
            .retry_after
            .and_then(|ra| HeaderValue::from_str(&ra.to_string()).ok())
        {
            headers.insert("Retry-After", val);
        }
    }
}

/// Axum 中间件函数
pub async fn rate_limit_middleware(
    State(mw): State<Arc<RateLimitMiddleware>>,
    request: Request,
    next: Next,
) -> Response {
    // 1. 在第一个 await 之前提取所有需要的数据
    // 这样我们就不会在跨 await 点时持有非 Sync 的 Request 引用
    let identifier = mw.extract_identifier(&request);
    let tier = mw.detect_user_tier(&request);
    let endpoint_type = mw.classifier.classify(request.uri(), request.method());

    // 2. 检查限流（异步）
    // 注意：不再将 &request 传递给异步函数
    // 检查是否启用限流
    if !mw.config_manager.is_enabled().await {
        return next.run(request).await;
    }

    // 获取限流规则
    let rule = mw
        .config_manager
        .get_rule(tier.as_str(), endpoint_type.as_str())
        .await;

    // 获取 Redis 键前缀
    let key_prefix = mw.config_manager.get_key_prefix().await;

    // 执行限流检查
    let result = mw.rate_limiter.check(&key_prefix, &identifier, &rule).await;

    if !result.allowed {
        warn!(
            %identifier,
            endpoint = %request.uri().path(),
            method = %request.method(),
            "Rate limit exceeded"
        );

        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Please try again later.",
        )
            .into_response();

        RateLimitMiddleware::add_rate_limit_headers(&mut response, &result);
        return response;
    }

    debug!(
        %identifier,
        endpoint = %request.uri().path(),
        method = %request.method(),
        count = result.count,
        remaining = result.remaining,
        "Request allowed"
    );

    // 允许请求通过
    let mut response = next.run(request).await;

    // 添加限流头
    RateLimitMiddleware::add_rate_limit_headers(&mut response, &result);

    response
}
