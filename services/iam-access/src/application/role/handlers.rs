//! 角色命令处理器

use std::sync::Arc;

use cuba_errors::{AppError, AppResult};

use crate::domain::role::{Role, RoleId, RoleRepository};
use super::commands::*;

use cuba_ports::EventPublisher;
use crate::domain::role::events::RbacEvent;

/// 角色命令处理器
pub struct RoleCommandHandler<R, EP>
where
    R: RoleRepository,
    EP: EventPublisher,
{
    role_repo: Arc<R>,
    event_publisher: Arc<EP>,
}

impl<R, EP> RoleCommandHandler<R, EP>
where
    R: RoleRepository,
    EP: EventPublisher,
{
    pub fn new(role_repo: Arc<R>, event_publisher: Arc<EP>) -> Self {
        Self { role_repo, event_publisher }
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

        // 保存 performed_by 用于事件
        let performed_by = cmd.performed_by;
        
        // 使用移动语义创建角色 (避免克隆)
        let role = cmd.into_role();

        self.role_repo.create(&role).await?;

        // 发布事件
        let event = RbacEvent::RoleCreated {
            id: role.id.0,
            tenant_id: role.tenant_id.0,
            code: role.code.clone(),
            name: role.name.clone(),
            by: performed_by,
        };
        self.event_publisher.publish("rbac.role.created", &event).await
            .map_err(|e| AppError::internal(e.to_string()))?;

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

        // 发布事件
        let event = RbacEvent::RoleUpdated {
            id: role.id.0,
            tenant_id: role.tenant_id.0,
            by: cmd.performed_by,
        };
        self.event_publisher.publish("rbac.role.updated", &event).await
            .map_err(|e| AppError::internal(e.to_string()))?;

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

        // 发布事件
        let event = RbacEvent::RoleDeleted {
            id: role.id.0,
            tenant_id: role.tenant_id.0,
            by: cmd.performed_by,
        };
        self.event_publisher.publish("rbac.role.deleted", &event).await
            .map_err(|e| AppError::internal(e.to_string()))?;

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

        // 发布事件
        let event = RbacEvent::RoleUpdated {
            id: role.id.0,
            tenant_id: role.tenant_id.0,
            by: None,
        };
        self.event_publisher.publish("rbac.role.updated", &event).await
            .map_err(|e| AppError::internal(e.to_string()))?;

        Ok(role)
    }
}
