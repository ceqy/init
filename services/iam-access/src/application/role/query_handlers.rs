//! 角色查询处理器

use std::sync::Arc;

use cuba_errors::{AppError, AppResult};

use super::queries::*;
use crate::domain::role::{Permission, Role, RoleId, RoleRepository, UserRoleRepository};

/// 角色查询结果
#[derive(Debug, Clone)]
pub struct RoleListResult {
    pub roles: Vec<Role>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

/// 角色查询处理器
pub struct RoleQueryHandler<R: RoleRepository, UR: UserRoleRepository> {
    role_repo: Arc<R>,
    user_role_repo: Arc<UR>,
}

impl<R: RoleRepository, UR: UserRoleRepository> RoleQueryHandler<R, UR> {
    pub fn new(role_repo: Arc<R>, user_role_repo: Arc<UR>) -> Self {
        Self {
            role_repo,
            user_role_repo,
        }
    }

    /// 获取角色详情
    pub async fn handle_get(&self, query: GetRoleQuery) -> AppResult<Role> {
        let role_id: RoleId = query
            .role_id
            .parse()
            .map_err(|_| AppError::validation("Invalid role ID"))?;

        self.role_repo
            .find_by_id(&role_id)
            .await?
            .ok_or_else(|| AppError::not_found("Role not found"))
    }

    /// 按代码获取角色
    pub async fn handle_get_by_code(&self, query: GetRoleByCodeQuery) -> AppResult<Role> {
        self.role_repo
            .find_by_code(&query.tenant_id, &query.code)
            .await?
            .ok_or_else(|| AppError::not_found("Role not found"))
    }

    /// 列出租户角色
    pub async fn handle_list(&self, query: ListRolesQuery) -> AppResult<RoleListResult> {
        let (roles, total) = self
            .role_repo
            .list_by_tenant(&query.tenant_id, query.page, query.page_size)
            .await?;

        Ok(RoleListResult {
            roles,
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }

    /// 搜索角色
    pub async fn handle_search(&self, query: SearchRolesQuery) -> AppResult<RoleListResult> {
        let (roles, total) = self
            .role_repo
            .search(&query.tenant_id, &query.query, query.page, query.page_size)
            .await?;

        Ok(RoleListResult {
            roles,
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }

    /// 获取用户角色
    pub async fn handle_get_user_roles(&self, query: GetUserRolesQuery) -> AppResult<Vec<Role>> {
        self.user_role_repo
            .get_user_roles(&query.user_id, &query.tenant_id)
            .await
    }

    /// 获取用户权限
    pub async fn handle_get_user_permissions(
        &self,
        query: GetUserPermissionsQuery,
    ) -> AppResult<Vec<Permission>> {
        self.user_role_repo
            .get_user_permissions(&query.user_id, &query.tenant_id)
            .await
    }

    /// 检查用户是否有某个权限
    pub async fn handle_check_user_permission(
        &self,
        query: CheckUserPermissionQuery,
    ) -> AppResult<bool> {
        self.user_role_repo
            .user_has_permission(&query.user_id, &query.tenant_id, &query.permission_code)
            .await
    }
}
