//! gRPC API 实现

mod auth_service_impl;

pub use auth_service_impl::*;

use cuba_errors::AppResult;

/// 创建认证服务
pub async fn create_auth_service() -> AppResult<AuthServiceImpl> {
    let service = AuthServiceImpl::new();
    Ok(service)
}
