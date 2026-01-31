//! 租户中间件 - 提取和验证租户信息

use std::sync::Arc;

use common::TenantId;
use tonic::{Request, Status};

use crate::domain::repositories::user::TenantRepository;

/// 租户上下文扩展
pub struct TenantContextExtension {
    pub tenant_id: TenantId,
}

/// 从请求中提取租户 ID
///
/// 优先级：
/// 1. JWT token 中的 tenant_id claim
/// 2. x-tenant-id header
/// 3. 返回错误
pub fn extract_tenant_id<T>(request: &Request<T>) -> Result<TenantId, Status> {
    // 1. 尝试从 JWT token 提取（如果已经通过认证中间件）
    if let Some(tenant_id) = request.extensions().get::<TenantId>() {
        return Ok(tenant_id.clone());
    }

    // 2. 尝试从 header 提取
    if let Some(tenant_id_str) = request.metadata().get("x-tenant-id") {
        let tenant_id_str = tenant_id_str
            .to_str()
            .map_err(|_| Status::invalid_argument("Invalid x-tenant-id header"))?;

        let uuid = uuid::Uuid::parse_str(tenant_id_str)
            .map_err(|_| Status::invalid_argument("Invalid tenant ID format"))?;

        return Ok(TenantId(uuid));
    }

    // 3. 租户 ID 缺失
    Err(Status::unauthenticated("Tenant ID is required"))
}

/// 租户验证中间件
///
/// 验证租户是否存在且激活
pub struct TenantValidationInterceptor {
    tenant_repo: Arc<dyn TenantRepository>,
}

impl TenantValidationInterceptor {
    pub fn new(tenant_repo: Arc<dyn TenantRepository>) -> Self {
        Self { tenant_repo }
    }

    /// 验证租户
    pub async fn validate_tenant(&self, tenant_id: &TenantId) -> Result<(), Status> {
        // 检查租户 ID 是否有效
        if tenant_id.0.is_nil() {
            return Err(Status::invalid_argument("Invalid tenant ID"));
        }

        // 从数据库查询租户
        let tenant = self
            .tenant_repo
            .find_by_id(tenant_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to find tenant: {}", e)))?
            .ok_or_else(|| Status::not_found("Tenant not found"))?;

        // 检查租户是否可用
        if !tenant.is_available() {
            return Err(Status::permission_denied("Tenant is not available"));
        }

        Ok(())
    }
}

