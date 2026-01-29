//! 限流中间件
//!
//! 集成用户等级检测、接口分类和限流检查

use crate::middleware::AuthContext;
use crate::rate_limit::classifier::EndpointClassifier;
use crate::rate_limit::config::ConfigManager;
use crate::rate_limit::limiter::RateLimiter;
use crate::rate_limit::types::{RateLimitResult, UserTier};
use axum::{
    extract::Request,
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

    /// 处理限流检查
    pub async fn check_rate_limit(&self, req: &Request) -> RateLimitResult {
        // 检查是否启用限流
        if !self.config_manager.is_enabled().await {
            return RateLimitResult {
                allowed: true,
                count: 0,
                remaining: u64::MAX,
                limit: u64::MAX,
                reset_at: 0,
                retry_after: None,
            };
        }

        // 提取客户端标识符
        let identifier = self.extract_identifier(req);

        // 检测用户等级
        let tier = self.detect_user_tier(req);

        // 分类接口类型
        let endpoint_type = self.classifier.classify(req.uri(), req.method());

        // 获取限流规则
        let rule = self
            .config_manager
            .get_rule(tier.as_str(), endpoint_type.as_str())
            .await;

        // 获取 Redis 键前缀
        let key_prefix = self.config_manager.get_key_prefix().await;

        // 执行限流检查
        self.rate_limiter
            .check(&key_prefix, &identifier, &rule)
            .await
    }

    /// 提取客户端标识符
    ///
    /// - 已认证用户: `user:{user_id}`
    /// - 未认证用户: `ip:{ip}`
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

    /// 检测用户等级
    fn detect_user_tier(&self, req: &Request) -> UserTier {
        UserTier::from_auth_context(req.extensions().get::<AuthContext>())
    }

    /// 为响应添加限流相关头
    fn add_rate_limit_headers(response: &mut Response, result: &RateLimitResult) {
        let headers = response.headers_mut();

        // X-RateLimit-Limit: 限制的最大请求数
        if let Ok(val) = HeaderValue::from_str(&result.limit.to_string()) {
            headers.insert("X-RateLimit-Limit", val);
        }

        // X-RateLimit-Remaining: 剩余请求数
        if let Ok(val) = HeaderValue::from_str(&result.remaining.to_string()) {
            headers.insert("X-RateLimit-Remaining", val);
        }

        // X-RateLimit-Reset: 窗口重置时间
        if let Ok(val) = HeaderValue::from_str(&result.reset_at.to_string()) {
            headers.insert("X-RateLimit-Reset", val);
        }

        // Retry-After: 建议重试等待时间（仅在拒绝时）
        if let Some(retry_after) = result.retry_after {
            if let Ok(val) = HeaderValue::from_str(&retry_after.to_string()) {
                headers.insert("Retry-After", val);
            }
        }
    }
}

/// Axum 中间件函数
pub async fn rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<Arc<RateLimitMiddleware>>,
    request: Request,
    next: Next,
) -> Response {
    // 检查限流
    let result = state.check_rate_limit(&request).await;

    if !result.allowed {
        // 限流触发，返回 429
        warn!(
            identifier = %state.extract_identifier(&request),
            endpoint = %request.uri().path(),
            method = %request.method(),
            "Rate limit exceeded"
        );

        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Please try again later.",
        )
            .into_response();

        // 添加限流头
        RateLimitMiddleware::add_rate_limit_headers(&mut response, &result);

        return response;
    }

    // 允许请求通过
    debug!(
        identifier = %state.extract_identifier(&request),
        endpoint = %request.uri().path(),
        method = %request.method(),
        count = result.count,
        remaining = result.remaining,
        "Request allowed"
    );

    // 继续处理请求
    let mut response = next.run(request).await;

    // 添加限流头
    RateLimitMiddleware::add_rate_limit_headers(&mut response, &result);

    response
}
