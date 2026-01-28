//! 中间件

use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use cuba_auth_core::{Claims, TokenService};
use tracing::{debug, warn, info};



/// 认证上下文
///
/// 包含已验证的 Claims 和原始 token
#[derive(Clone, Debug)]
pub struct AuthContext {
    pub claims: Claims,
    pub token: String,
}

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
            .get::<AuthContext>()
            .map(|ctx| AuthClaims(ctx.claims.clone()))
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing claims in request extensions (auth_middleware may not have run)",
            ))
    }
}

/// 认证上下文提取器
///
/// 用于从请求中获取 Claims 和原始 token
pub struct AuthToken(pub AuthContext);

impl<S> FromRequestParts<S> for AuthToken
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
            .get::<AuthContext>()
            .cloned()
            .map(AuthToken)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing auth context in request extensions",
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

    // Reworking the logic to handle lifetimes correctly
    let query_token_owner: Option<String> = request
        .uri()
        .query()
        .and_then(|query| {
            url::form_urlencoded::parse(query.as_bytes())
                .find(|(key, _)| key == "token")
                .map(|(_, value)| value.to_string())
        });
        
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => Some(&header[7..]),
        _ => query_token_owner.as_deref(),
    };

    if let Some(token) = token {
            debug!("Validating JWT token");

            match token_service.validate_token(token) {
                Ok(claims) => {
                    info!(
                        user_id = %claims.sub,
                        tenant_id = %claims.tenant_id,
                        "Token validated successfully"
                    );

                    // 将 AuthContext (claims + token) 注入到请求扩展中
                    let auth_context = AuthContext {
                        claims,
                        token: token.to_string(),
                    };
                    let mut request = request;
                    request.extensions_mut().insert(auth_context);

                    Ok(next.run(request).await)
                }
                Err(e) => {
                    warn!(error = %e, "Token validation failed");
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
    } else {
            warn!("Missing or invalid authorization header/query param");
            Err(StatusCode::UNAUTHORIZED)
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
