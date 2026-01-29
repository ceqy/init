//! gRPC Interceptors

use cuba_auth_core::{Claims, TokenService};
use cuba_errors::AppError;
use tonic::{Request, Status};

/// 认证拦截器
#[allow(clippy::result_large_err)]
pub fn auth_interceptor(
    token_service: &TokenService,
    mut request: Request<()>,
) -> Result<Request<()>, Status> {
    let token = extract_token(&request)?;

    let claims = token_service
        .validate_token(&token)
        .map_err(|e| Status::unauthenticated(e.to_string()))?;

    request.extensions_mut().insert(claims);

    Ok(request)
}

/// 从请求中提取 token
#[allow(clippy::result_large_err)]
fn extract_token<T>(request: &Request<T>) -> Result<String, Status> {
    let auth_header = request
        .metadata()
        .get("authorization")
        .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| Status::unauthenticated("Invalid authorization header"))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(Status::unauthenticated("Invalid authorization scheme"));
    }

    Ok(auth_str[7..].to_string())
}

/// 从请求扩展中获取 Claims
pub fn get_claims<T>(request: &Request<T>) -> Result<&Claims, AppError> {
    request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::unauthorized("No claims found in request"))
}
