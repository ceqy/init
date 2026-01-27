//! 中间件

use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use cuba_auth_core::{Claims, TokenService};
use tracing::{debug, warn, info};



/// 认证 Claims 提取器
///
/// 用于从请求中获取已验证的 Claims
/// 应该在 auth_middleware 之后使用
pub struct AuthClaims(pub Claims);

impl<S> FromRequestParts<S> for AuthClaims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(AuthClaims)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing claims in request extensions (auth_middleware may not have run)",
            ))
    }
}

/// JWT 认证中间件
///
/// 验证请求中的 JWT token 并将 claims 注入到请求扩展中
pub async fn auth_middleware(
    State(token_service): State<TokenService>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..];
            debug!("Validating JWT token");

            match token_service.validate_token(token) {
                Ok(claims) => {
                    info!(
                        user_id = %claims.sub,
                        tenant_id = %claims.tenant_id,
                        "Token validated successfully"
                    );

                    // 将 claims 注入到请求扩展中
                    let mut request = request;
                    request.extensions_mut().insert(claims);

                    Ok(next.run(request).await)
                }
                Err(e) => {
                    warn!(error = %e, "Token validation failed");
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        }
        _ => {
            warn!("Missing or invalid authorization header");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
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
    use cuba_common::{UserId, TenantId};
    use tower::ServiceExt;

    async fn handler() -> impl axum::response::IntoResponse {
        "OK"
    }

    #[tokio::test]
    async fn test_auth_middleware_valid_token() {
        let secret = "test_secret";
        let token_service = TokenService::new(secret, 3600, 3600);
        let user_id = UserId::new();
        let tenant_id = TenantId::new();
        let token = token_service
            .generate_access_token(&user_id, &tenant_id, vec![], vec![])
            .unwrap();

        let app = Router::new()
            .route("/", get(handler))
            .layer(middleware::from_fn_with_state(token_service.clone(), auth_middleware))
            .with_state(token_service);

        let req = Request::builder()
            .uri("/")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_invalid_token() {
        let secret = "test_secret";
        let token_service = TokenService::new(secret, 3600, 3600);
        
        let app = Router::new()
            .route("/", get(handler))
            .layer(middleware::from_fn_with_state(token_service.clone(), auth_middleware))
            .with_state(token_service);

        let req = Request::builder()
            .uri("/")
            .header("Authorization", "Bearer invalid_token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_missing_header() {
        let secret = "test_secret";
        let token_service = TokenService::new(secret, 3600, 3600);
        
        let app = Router::new()
            .route("/", get(handler))
            .layer(middleware::from_fn_with_state(token_service.clone(), auth_middleware))
            .with_state(token_service);

        let req = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_expired_token() {
        let secret = "test_secret";
        // Create a token service with very short expiration
        let token_service = TokenService::new(secret, -3600, -3600); 
        let user_id = UserId::new();
        let tenant_id = TenantId::new();
        // Generate a token that is already expired
        let token = token_service
            .generate_access_token(&user_id, &tenant_id, vec![], vec![])
            .unwrap();

        let app = Router::new()
            .route("/", get(handler))
            .layer(middleware::from_fn_with_state(token_service.clone(), auth_middleware))
            .with_state(token_service);

        let req = Request::builder()
            .uri("/")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_wrong_secret() {
        let secret = "correct_secret";
        let token_service = TokenService::new(secret, 3600, 3600);
        
        let wrong_secret_service = TokenService::new("wrong_secret", 3600, 3600);
        let user_id = UserId::new();
        let tenant_id = TenantId::new();
        let token = wrong_secret_service
            .generate_access_token(&user_id, &tenant_id, vec![], vec![])
            .unwrap();

        let app = Router::new()
            .route("/", get(handler))
            .layer(middleware::from_fn_with_state(token_service.clone(), auth_middleware))
            .with_state(token_service);

        let req = Request::builder()
            .uri("/")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
