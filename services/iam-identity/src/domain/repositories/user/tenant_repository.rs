//! 租户 Repository trait

use async_trait::async_trait;
use cuba_common::TenantId;
use cuba_errors::AppResult;

use crate::shared::domain::entities::{Tenant, TenantStatus};

/// 租户仓储接口
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// 根据 ID 查找租户
    async fn find_by_id(&self, id: &TenantId) -> AppResult<Option<Tenant>>;

    /// 根据名称查找租户
    async fn find_by_name(&self, name: &str) -> AppResult<Option<Tenant>>;

    /// 根据域名查找租户
    async fn find_by_domain(&self, domain: &str) -> AppResult<Option<Tenant>>;

    /// 保存租户
    async fn save(&self, tenant: &Tenant) -> AppResult<()>;

    /// 更新租户
    async fn update(&self, tenant: &Tenant) -> AppResult<()>;

    /// 删除租户（软删除）
    async fn delete(&self, id: &TenantId) -> AppResult<()>;

    /// 检查租户名称是否存在
    async fn exists_by_name(&self, name: &str) -> AppResult<bool>;

    /// 检查域名是否存在
    async fn exists_by_domain(&self, domain: &str) -> AppResult<bool>;

    /// 分页查询租户列表
    async fn list(
        &self,
        status: Option<TenantStatus>,
        search: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> AppResult<(Vec<Tenant>, i64)>;

    /// 统计租户数量
    async fn count(&self) -> AppResult<i64>;

    /// 查找即将过期的试用租户
    async fn find_expiring_trials(&self, days: i64) -> AppResult<Vec<Tenant>>;

    /// 查找即将过期的订阅租户
    async fn find_expiring_subscriptions(&self, days: i64) -> AppResult<Vec<Tenant>>;
}
