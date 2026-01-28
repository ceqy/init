//! 角色命令处理器

use std::sync::Arc;

use cuba_errors::{AppError, AppResult};

use crate::domain::role::{Role, RoleId, RoleRepository};
use super::commands::*;

/// 角色命令处理器
pub struct RoleCommandHandler<R: RoleRepository> {
    role_repo: Arc<R>,
}

impl<R: RoleRepository> RoleCommandHandler<R> {
    pub fn new(role_repo: Arc<R>) -> Self {
        Self { role_repo }
    }

    /// 创建角色
    pub async fn handle_create(&self, cmd: CreateRoleCommand) -> AppResult<Role> {
        // 检查代码是否已存在
        if self.role_repo.exists_by_code(&cmd.tenant_id, &cmd.code).await? {
            return Err(AppError::conflict(format!(
                "Role with code '{}' already exists",
                cmd.code
            )));
        }

        let role = if cmd.is_system {
            Role::system_role(cmd.tenant_id, cmd.code, cmd.name, cmd.description)
        } else {
            Role::new(cmd.tenant_id, cmd.code, cmd.name, cmd.description)
        };

        self.role_repo.create(&role).await?;

        Ok(role)
    }

    /// 更新角色
    pub async fn handle_update(&self, cmd: UpdateRoleCommand) -> AppResult<Role> {
        let role_id: RoleId = cmd.role_id.parse()
            .map_err(|_| AppError::validation("Invalid role ID"))?;

        let mut role = self.role_repo.find_by_id(&role_id)
            .await?
            .ok_or_else(|| AppError::not_found("Role not found"))?;

        role.update(cmd.name, cmd.description);
        self.role_repo.update(&role).await?;

        Ok(role)
    }

    /// 删除角色
    pub async fn handle_delete(&self, cmd: DeleteRoleCommand) -> AppResult<()> {
        let role_id: RoleId = cmd.role_id.parse()
            .map_err(|_| AppError::validation("Invalid role ID"))?;

        let role = self.role_repo.find_by_id(&role_id)
            .await?
            .ok_or_else(|| AppError::not_found("Role not found"))?;

        // 系统角色不可删除
        if role.is_system {
            return Err(AppError::forbidden("System role cannot be deleted"));
        }

        self.role_repo.delete(&role_id).await?;

        Ok(())
    }

    /// 激活/停用角色
    pub async fn handle_set_active(&self, cmd: SetRoleActiveCommand) -> AppResult<Role> {
        let role_id: RoleId = cmd.role_id.parse()
            .map_err(|_| AppError::validation("Invalid role ID"))?;

        let mut role = self.role_repo.find_by_id(&role_id)
            .await?
            .ok_or_else(|| AppError::not_found("Role not found"))?;

        if cmd.is_active {
            role.activate();
        } else {
            role.deactivate();
        }

        self.role_repo.update(&role).await?;

        Ok(role)
    }
}
