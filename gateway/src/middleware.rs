//! 中间件

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use cuba_auth_core::TokenService;
use tracing::warn;

/// JWT 认证中间件
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let _token = &header[7..];
            // TODO: 验证 token
            Ok(next.run(request).await)
        }
        _ => {
            warn!("Missing or invalid authorization header");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// 请求日志中间件
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        "Request completed"
    );

    response
}
