//! 用户 Repository trait

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;

use crate::domain::user::User;
use crate::domain::value_objects::{Email, Username};

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// 根据 ID 查找用户（带租户隔离）
    async fn find_by_id(&self, id: &UserId, tenant_id: &TenantId) -> AppResult<Option<User>>;

    /// 根据用户名查找用户（带租户隔离）
    async fn find_by_username(
        &self,
        username: &Username,
        tenant_id: &TenantId,
    ) -> AppResult<Option<User>>;

    /// 根据邮箱查找用户（带租户隔离）
    async fn find_by_email(&self, email: &Email, tenant_id: &TenantId) -> AppResult<Option<User>>;

    /// 保存用户（自动使用用户的 tenant_id）
    async fn save(&self, user: &User) -> AppResult<()>;

    /// 更新用户（验证 tenant_id 匹配）
    async fn update(&self, user: &User) -> AppResult<()>;

    /// 删除用户（带租户隔离）
    async fn delete(&self, id: &UserId, tenant_id: &TenantId) -> AppResult<()>;

    /// 检查用户名是否存在（带租户隔离）
    async fn exists_by_username(
        &self,
        username: &Username,
        tenant_id: &TenantId,
    ) -> AppResult<bool>;

    /// 检查邮箱是否存在（带租户隔离）
    async fn exists_by_email(&self, email: &Email, tenant_id: &TenantId) -> AppResult<bool>;

    /// 分页查询用户列表（强制租户隔离）
    async fn list(
        &self,
        tenant_id: &TenantId,
        status: Option<&str>,
        search: Option<&str>,
        role_ids: &[String],
        page: i32,
        page_size: i32,
    ) -> AppResult<(Vec<User>, i64)>;

    /// 统计租户的用户数量
    async fn count_by_tenant(&self, tenant_id: &TenantId) -> AppResult<i64>;
}
