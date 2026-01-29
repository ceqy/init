//! 策略仓储接口

use async_trait::async_trait;
use cuba_common::TenantId;
use cuba_errors::AppResult;

use super::policy::{Policy, PolicyId};

/// 策略仓储接口
#[async_trait]
pub trait PolicyRepository: Send + Sync {
    /// 创建策略
    async fn create(&self, policy: &Policy) -> AppResult<()>;

    /// 更新策略
    async fn update(&self, policy: &Policy) -> AppResult<()>;

    /// 删除策略
    async fn delete(&self, id: &PolicyId) -> AppResult<()>;

    /// 根据 ID 查找策略
    async fn find_by_id(&self, id: &PolicyId) -> AppResult<Option<Policy>>;

    /// 列出租户下的所有策略
    async fn list_by_tenant(
        &self,
        tenant_id: &TenantId,
        page: u32,
        page_size: u32,
    ) -> AppResult<(Vec<Policy>, i64)>;

    /// 列出所有激活的策略 (用于评估)
    async fn list_active_by_tenant(&self, tenant_id: &TenantId) -> AppResult<Vec<Policy>>;

    /// 按主体查找相关策略
    async fn find_by_subject(&self, tenant_id: &TenantId, subject: &str) -> AppResult<Vec<Policy>>;

    /// 按资源查找相关策略
    async fn find_by_resource(
        &self,
        tenant_id: &TenantId,
        resource: &str,
    ) -> AppResult<Vec<Policy>>;

    /// 检查策略名称是否存在
    async fn exists_by_name(&self, tenant_id: &TenantId, name: &str) -> AppResult<bool>;
}
