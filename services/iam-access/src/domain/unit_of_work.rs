//! Unit of Work 模式
//!
//! 提供跨多个 Repository 的事务协调能力，确保操作的原子性。

use async_trait::async_trait;
use cuba_errors::AppResult;

use crate::domain::policy::PolicyRepository;
use crate::domain::role::{
    PermissionRepository, RolePermissionRepository, RoleRepository, UserRoleRepository,
};
use crate::infrastructure::persistence::outbox_repository::OutboxRepository;

/// Unit of Work trait
///
/// 协调多个 Repository 在同一事务中的操作。
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// 获取角色 Repository
    fn roles(&self) -> &dyn RoleRepository;

    /// 获取权限 Repository
    fn permissions(&self) -> &dyn PermissionRepository;

    /// 获取角色权限 Repository
    fn role_permissions(&self) -> &dyn RolePermissionRepository;

    /// 获取用户角色 Repository
    fn user_roles(&self) -> &dyn UserRoleRepository;

    /// 获取策略 Repository
    fn policies(&self) -> &dyn PolicyRepository;

    /// 获取 Outbox Repository
    fn outbox(&self) -> &dyn OutboxRepository;

    /// 在当前事务中保存 Outbox 事件
    async fn save_outbox_event(
        &self,
        aggregate_type: &str,
        aggregate_id: uuid::Uuid,
        event_type: &str,
        payload_json: &str,
    ) -> AppResult<uuid::Uuid>;

    /// 提交事务
    async fn commit(self: Box<Self>) -> AppResult<()>;

    /// 回滚事务
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}

/// Unit of Work 工厂 trait
#[async_trait]
pub trait UnitOfWorkFactory: Send + Sync {
    /// 开始新的事务
    async fn begin(&self) -> AppResult<Box<dyn UnitOfWork>>;
}
