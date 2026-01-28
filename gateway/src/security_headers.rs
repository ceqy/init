//! 安全响应头中间件

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

/// 添加安全响应头的中间件
///
/// 添加以下安全头：
/// - Strict-Transport-Security (HSTS): 强制使用 HTTPS
/// - X-Frame-Options: 防止点击劫持
/// - X-Content-Type-Options: 防止 MIME 类型嗅探
/// - X-XSS-Protection: 启用浏览器 XSS 过滤器
/// - Content-Security-Policy: 内容安全策略
/// - Referrer-Policy: 控制 Referer 头信息
/// - Permissions-Policy: 控制浏览器功能权限
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // HSTS: 强制使用 HTTPS，有效期 1 年，包括子域名
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );

    // X-Frame-Options: 防止页面被嵌入到 iframe 中
    headers.insert(
        "X-Frame-Options",
        "DENY".parse().unwrap(),
    );

    // X-Content-Type-Options: 防止浏览器进行 MIME 类型嗅探
    headers.insert(
        "X-Content-Type-Options",
        "nosniff".parse().unwrap(),
    );

    // X-XSS-Protection: 启用浏览器的 XSS 过滤器
    headers.insert(
        "X-XSS-Protection",
        "1; mode=block".parse().unwrap(),
    );

    // Content-Security-Policy: 内容安全策略
    // 这是一个相对宽松的策略，生产环境应该根据实际需求调整
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' ws: wss:; frame-ancestors 'none'".parse().unwrap(),
    );

    // Referrer-Policy: 控制 Referer 头的发送
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    // Permissions-Policy: 控制浏览器功能权限
    headers.insert(
        "Permissions-Policy",
        "geolocation=(), microphone=(), camera=()".parse().unwrap(),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
        middleware,
    };
    use tower::ServiceExt;

    async fn handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_security_headers_added() {
        let app = Router::new()
            .route("/", get(handler))
            .layer(middleware::from_fn(security_headers_middleware));

        let req = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();

        // 验证所有安全头都已添加
        assert!(headers.contains_key("Strict-Transport-Security"));
        assert!(headers.contains_key("X-Frame-Options"));
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("X-XSS-Protection"));
        assert!(headers.contains_key("Content-Security-Policy"));
        assert!(headers.contains_key("Referrer-Policy"));
        assert!(headers.contains_key("Permissions-Policy"));

        // 验证具体值
        assert_eq!(
            headers.get("X-Frame-Options").unwrap(),
            "DENY"
        );
        assert_eq!(
            headers.get("X-Content-Type-Options").unwrap(),
            "nosniff"
        );
    }
}
