//! 角色命令处理器

use super::commands::*;
use crate::domain::role::events::RbacEvent;
use crate::domain::role::{Role, RoleId};
use crate::domain::unit_of_work::UnitOfWorkFactory;
use cuba_errors::{AppError, AppResult};
use std::sync::Arc;
use uuid::Uuid;

/// 角色命令处理器
pub struct RoleCommandHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl RoleCommandHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }

    /// 创建角色
    pub async fn handle_create(&self, cmd: CreateRoleCommand) -> AppResult<Role> {
        // 验证输入
        cmd.validate().map_err(|e| AppError::validation(e))?;

        let uow = self.uow_factory.begin().await?;

        // 检查代码是否已存在
        if uow
            .roles()
            .exists_by_code(&cmd.tenant_id, &cmd.code)
            .await?
        {
            return Err(AppError::conflict(format!(
                "Role with code '{}' already exists",
                cmd.code
            )));
        }

        // 保存 performed_by 用于事件
        let performed_by = cmd.performed_by.clone();

        // 使用移动语义创建角色 (避免克隆)
        let role = cmd.into_role();

        uow.roles().create(&role).await?;

        // 写入 Outbox 事件
        let event = RbacEvent::RoleCreated {
            id: role.id.0,
            tenant_id: role.tenant_id.0,
            code: role.code.clone(),
            name: role.name.clone(),
            by: performed_by,
        };

        let payload = serde_json::to_string(&event)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;

        uow.save_outbox_event("Role", role.id.0, "rbac.role.created", &payload)
            .await?;

        uow.commit().await?;

        Ok(role)
    }

    /// 更新角色
    pub async fn handle_update(&self, cmd: UpdateRoleCommand) -> AppResult<Role> {
        let role_id: RoleId = cmd
            .role_id
            .parse()
            .map_err(|_| AppError::validation("Invalid role ID"))?;

        let uow = self.uow_factory.begin().await?;

        let mut role = uow
            .roles()
            .find_by_id(&role_id)
            .await?
            .ok_or_else(|| AppError::not_found("Role not found"))?;

        role.update(cmd.name, cmd.description);
        uow.roles().update(&role).await?;

        // 写入 Outbox 事件
        let event = RbacEvent::RoleUpdated {
            id: role.id.0,
            tenant_id: role.tenant_id.0,
            by: cmd.performed_by.clone(),
        };

        let payload = serde_json::to_string(&event)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;

        uow.save_outbox_event("Role", role.id.0, "rbac.role.updated", &payload)
            .await?;

        uow.commit().await?;

        Ok(role)
    }

    /// 删除角色
    pub async fn handle_delete(&self, cmd: DeleteRoleCommand) -> AppResult<()> {
        let role_id: RoleId = cmd
            .role_id
            .parse()
            .map_err(|_| AppError::validation("Invalid role ID"))?;

        let uow = self.uow_factory.begin().await?;

        let role = uow
            .roles()
            .find_by_id(&role_id)
            .await?
            .ok_or_else(|| AppError::not_found("Role not found"))?;

        // 系统角色不可删除
        if role.is_system {
            return Err(AppError::forbidden("System role cannot be deleted"));
        }

        uow.roles().delete(&role_id).await?;

        // 写入 Outbox 事件
        let event = RbacEvent::RoleDeleted {
            id: role.id.0,
            tenant_id: role.tenant_id.0,
            by: cmd.performed_by.clone(),
        };

        let payload = serde_json::to_string(&event)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;

        uow.save_outbox_event("Role", role.id.0, "rbac.role.deleted", &payload)
            .await?;

        uow.commit().await?;

        Ok(())
    }

    /// 激活/停用角色
    pub async fn handle_set_active(&self, cmd: SetRoleActiveCommand) -> AppResult<Role> {
        let role_id: RoleId = cmd
            .role_id
            .parse()
            .map_err(|_| AppError::validation("Invalid role ID"))?;

        let uow = self.uow_factory.begin().await?;

        let mut role = uow
            .roles()
            .find_by_id(&role_id)
            .await?
            .ok_or_else(|| AppError::not_found("Role not found"))?;

        if cmd.is_active {
            role.activate();
        } else {
            role.deactivate();
        }

        uow.roles().update(&role).await?;

        // 写入 Outbox 事件
        let event = RbacEvent::RoleUpdated {
            id: role.id.0,
            tenant_id: role.tenant_id.0,
            by: None,
        };

        let payload = serde_json::to_string(&event)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;

        uow.save_outbox_event("Role", role.id.0, "rbac.role.updated", &payload)
            .await?;

        uow.commit().await?;

        Ok(role)
    }
}
