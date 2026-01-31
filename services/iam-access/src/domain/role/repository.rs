//! 角色仓储接口

use async_trait::async_trait;
use common::TenantId;
use errors::AppResult;

use super::permission::{Permission, PermissionId};
use super::role::{Role, RoleId};

/// 角色仓储接口
#[async_trait]
pub trait RoleRepository: Send + Sync {
    /// 创建角色
    async fn create(&self, role: &Role) -> AppResult<()>;

    /// 更新角色
    async fn update(&self, role: &Role) -> AppResult<()>;

    /// 删除角色
    async fn delete(&self, id: &RoleId) -> AppResult<()>;

    /// 根据 ID 查找角色
    async fn find_by_id(&self, id: &RoleId) -> AppResult<Option<Role>>;

    /// 根据代码查找角色
    async fn find_by_code(&self, tenant_id: &TenantId, code: &str) -> AppResult<Option<Role>>;

    /// 列出租户下的所有角色
    async fn list_by_tenant(
        &self,
        tenant_id: &TenantId,
        page: u32,
        page_size: u32,
    ) -> AppResult<(Vec<Role>, i64)>;

    /// 搜索角色
    async fn search(
        &self,
        tenant_id: &TenantId,
        query: &str,
        page: u32,
        page_size: u32,
    ) -> AppResult<(Vec<Role>, i64)>;

    /// 检查角色代码是否存在
    async fn exists_by_code(&self, tenant_id: &TenantId, code: &str) -> AppResult<bool>;

    /// 获取所有活跃租户 ID (用于缓存预热)
    async fn list_active_tenants(&self) -> AppResult<Vec<TenantId>>;
}

/// 权限仓储接口
#[async_trait]
pub trait PermissionRepository: Send + Sync {
    /// 创建权限
    async fn create(&self, permission: &Permission) -> AppResult<()>;

    /// 更新权限
    async fn update(&self, permission: &Permission) -> AppResult<()>;

    /// 删除权限
    async fn delete(&self, id: &PermissionId) -> AppResult<()>;

    /// 根据 ID 查找权限
    async fn find_by_id(&self, id: &PermissionId) -> AppResult<Option<Permission>>;

    /// 根据代码查找权限
    async fn find_by_code(&self, code: &str) -> AppResult<Option<Permission>>;

    /// 根据多个 ID 批量查找权限
    async fn find_by_ids(&self, ids: &[PermissionId]) -> AppResult<Vec<Permission>>;

    /// 列出所有权限
    async fn list_all(&self, page: u32, page_size: u32) -> AppResult<(Vec<Permission>, i64)>;

    /// 按模块列出权限
    async fn list_by_module(&self, module: &str) -> AppResult<Vec<Permission>>;

    /// 按资源列出权限
    async fn list_by_resource(&self, resource: &str) -> AppResult<Vec<Permission>>;

    /// 检查权限代码是否存在
    async fn exists_by_code(&self, code: &str) -> AppResult<bool>;
}

/// 角色权限关联仓储接口
#[async_trait]
pub trait RolePermissionRepository: Send + Sync {
    /// 为角色分配权限
    async fn assign_permissions(
        &self,
        role_id: &RoleId,
        permission_ids: &[PermissionId],
    ) -> AppResult<()>;

    /// 移除角色的权限
    async fn remove_permissions(
        &self,
        role_id: &RoleId,
        permission_ids: &[PermissionId],
    ) -> AppResult<()>;

    /// 获取角色的所有权限
    async fn get_role_permissions(&self, role_id: &RoleId) -> AppResult<Vec<Permission>>;

    /// 清空角色的所有权限
    async fn clear_role_permissions(&self, role_id: &RoleId) -> AppResult<()>;
}

/// 用户角色关联仓储接口
#[async_trait]
pub trait UserRoleRepository: Send + Sync {
    /// 为用户分配角色
    async fn assign_roles(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
        role_ids: &[RoleId],
    ) -> AppResult<()>;

    /// 移除用户的角色
    async fn remove_roles(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
        role_ids: &[RoleId],
    ) -> AppResult<()>;

    /// 获取用户的所有角色
    async fn get_user_roles(&self, user_id: &str, tenant_id: &TenantId) -> AppResult<Vec<Role>>;

    /// 获取用户的所有权限 (聚合所有角色的权限)
    async fn get_user_permissions(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<Permission>>;

    /// 清空用户的所有角色
    async fn clear_user_roles(&self, user_id: &str, tenant_id: &TenantId) -> AppResult<()>;

    /// 检查用户是否拥有某个角色
    async fn user_has_role(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
        role_id: &RoleId,
    ) -> AppResult<bool>;

    /// 检查用户是否拥有某个权限
    async fn user_has_permission(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
        permission_code: &str,
    ) -> AppResult<bool>;
}
