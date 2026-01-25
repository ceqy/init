//! 租户中间件 - 提取和验证租户信息

use std::sync::Arc;

use cuba_common::TenantId;
use tonic::{Request, Status};

use crate::shared::domain::repositories::TenantRepository;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataValue;

    #[test]
    fn test_extract_tenant_id_from_header() {
        let tenant_id = TenantId::new();
        let mut request = Request::new(());

        // 添加 header
        let metadata_value = MetadataValue::try_from(tenant_id.0.to_string()).unwrap();
        request.metadata_mut().insert("x-tenant-id", metadata_value);

        let extracted = extract_tenant_id(&request).unwrap();
        assert_eq!(extracted.0, tenant_id.0);
    }

    #[test]
    fn test_extract_tenant_id_from_extension() {
        let tenant_id = TenantId::new();
        let mut request = Request::new(());

        // 添加到 extensions
        request.extensions_mut().insert(tenant_id.clone());

        let extracted = extract_tenant_id(&request).unwrap();
        assert_eq!(extracted.0, tenant_id.0);
    }

    #[test]
    fn test_extract_tenant_id_missing() {
        let request = Request::new(());
        let result = extract_tenant_id(&request);
        assert!(result.is_err());
    }
}
