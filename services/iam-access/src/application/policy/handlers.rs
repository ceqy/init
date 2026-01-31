//! 策略命令处理器

use std::sync::Arc;

use errors::{AppError, AppResult};

use super::commands::*;
use crate::domain::policy::{Effect, Policy, PolicyId, events::PolicyEvent};
use crate::domain::unit_of_work::UnitOfWorkFactory;

/// 策略命令处理器
pub struct PolicyCommandHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl PolicyCommandHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }

    /// 创建策略
    pub async fn handle_create(&self, cmd: CreatePolicyCommand) -> AppResult<Policy> {
        let uow = self.uow_factory.begin().await?;

        // 检查名称是否已存在
        if uow
            .policies()
            .exists_by_name(&cmd.tenant_id, &cmd.name)
            .await?
        {
            return Err(AppError::conflict(format!(
                "Policy with name '{}' already exists",
                cmd.name
            )));
        }

        let effect: Effect = cmd
            .effect
            .parse()
            .map_err(|_| AppError::validation("Invalid effect, must be 'ALLOW' or 'DENY'"))?;

        let policy = Policy::new(
            cmd.tenant_id,
            cmd.name,
            effect,
            cmd.subjects,
            cmd.resources,
            cmd.actions,
        )
        .with_priority(cmd.priority);

        let policy = if let Some(desc) = cmd.description {
            policy.with_description(desc)
        } else {
            policy
        };

        let policy = if let Some(cond) = cmd.conditions {
            policy.with_conditions(cond)
        } else {
            policy
        };

        uow.policies().create(&policy).await?;

        // 记录 Outbox 事件
        let event = PolicyEvent::created(&policy, &cmd.performed_by);
        let payload = serde_json::to_string(&event)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;

        uow.save_outbox_event("Policy", policy.id.0, "rbac.policy.created", &payload)
            .await?;

        uow.commit().await?;

        Ok(policy)
    }

    /// 更新策略
    pub async fn handle_update(&self, cmd: UpdatePolicyCommand) -> AppResult<Policy> {
        let uow = self.uow_factory.begin().await?;

        let policy_id: PolicyId = cmd
            .policy_id
            .parse()
            .map_err(|_| AppError::validation("Invalid policy ID"))?;

        let mut policy = uow
            .policies()
            .find_by_id(&policy_id)
            .await?
            .ok_or_else(|| AppError::not_found("Policy not found"))?;

        let effect: Effect = cmd
            .effect
            .parse()
            .map_err(|_| AppError::validation("Invalid effect, must be 'ALLOW' or 'DENY'"))?;

        policy.name = cmd.name;
        policy.description = cmd.description;
        policy.effect = effect;
        policy.subjects = cmd.subjects;
        policy.resources = cmd.resources;
        policy.actions = cmd.actions;
        policy.conditions = cmd.conditions;
        policy.priority = cmd.priority;

        uow.policies().update(&policy).await?;

        // 记录 Outbox 事件
        let event = PolicyEvent::updated(&policy, &cmd.performed_by);
        let payload = serde_json::to_string(&event)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;

        uow.save_outbox_event("Policy", policy.id.0, "rbac.policy.updated", &payload)
            .await?;

        uow.commit().await?;

        Ok(policy)
    }

    /// 删除策略
    pub async fn handle_delete(&self, cmd: DeletePolicyCommand) -> AppResult<()> {
        let uow = self.uow_factory.begin().await?;

        let policy_id: PolicyId = cmd
            .policy_id
            .parse()
            .map_err(|_| AppError::validation("Invalid policy ID"))?;

        // 检查策略是否存在
        let policy = uow
            .policies()
            .find_by_id(&policy_id)
            .await?
            .ok_or_else(|| AppError::not_found("Policy not found"))?;

        uow.policies().delete(&policy_id).await?;

        // 记录 Outbox 事件
        let event = PolicyEvent::deleted(
            policy.id.clone(),
            policy.tenant_id.clone(),
            &cmd.performed_by,
        );
        let payload = serde_json::to_string(&event)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;

        uow.save_outbox_event("Policy", policy.id.0, "rbac.policy.deleted", &payload)
            .await?;

        uow.commit().await?;

        Ok(())
    }

    /// 激活/停用策略
    pub async fn handle_set_active(&self, cmd: SetPolicyActiveCommand) -> AppResult<Policy> {
        let uow = self.uow_factory.begin().await?;

        let policy_id: PolicyId = cmd
            .policy_id
            .parse()
            .map_err(|_| AppError::validation("Invalid policy ID"))?;

        let mut policy = uow
            .policies()
            .find_by_id(&policy_id)
            .await?
            .ok_or_else(|| AppError::not_found("Policy not found"))?;

        if cmd.is_active {
            policy.activate();
        } else {
            policy.deactivate();
        }

        uow.policies().update(&policy).await?;

        // 记录 Outbox 事件
        let event = PolicyEvent::updated(&policy, &cmd.performed_by);
        let payload = serde_json::to_string(&event)
            .map_err(|e| AppError::internal(format!("Failed to serialize event: {}", e)))?;

        uow.save_outbox_event("Policy", policy.id.0, "rbac.policy.updated", &payload)
            .await?;

        uow.commit().await?;

        Ok(policy)
    }
}
