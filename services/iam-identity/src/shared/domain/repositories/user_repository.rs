//! 用户 Repository trait

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;

use crate::shared::domain::entities::User;
use crate::shared::domain::value_objects::{Email, Username};

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// 根据 ID 查找用户
    async fn find_by_id(&self, id: &UserId) -> AppResult<Option<User>>;

    /// 根据用户名查找用户
    async fn find_by_username(&self, username: &Username, tenant_id: &TenantId) -> AppResult<Option<User>>;

    /// 根据邮箱查找用户
    async fn find_by_email(&self, email: &Email, tenant_id: &TenantId) -> AppResult<Option<User>>;

    /// 保存用户
    async fn save(&self, user: &User) -> AppResult<()>;

    /// 更新用户
    async fn update(&self, user: &User) -> AppResult<()>;

    /// 删除用户
    async fn delete(&self, id: &UserId) -> AppResult<()>;

    /// 检查用户名是否存在
    async fn exists_by_username(&self, username: &Username, tenant_id: &TenantId) -> AppResult<bool>;

    /// 检查邮箱是否存在
    async fn exists_by_email(&self, email: &Email, tenant_id: &TenantId) -> AppResult<bool>;

    /// 分页查询用户列表
    async fn list(
        &self,
        tenant_id: Option<&TenantId>,
        status: Option<&str>,
        search: Option<&str>,
        role_ids: &[String],
        page: i32,
        page_size: i32,
    ) -> AppResult<(Vec<User>, i64)>;
}
