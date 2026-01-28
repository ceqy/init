//! 策略命令处理器

use std::sync::Arc;

use cuba_errors::{AppError, AppResult};

use crate::domain::policy::{Effect, Policy, PolicyId, PolicyRepository};
use super::commands::*;

/// 策略命令处理器
pub struct PolicyCommandHandler<R: PolicyRepository> {
    policy_repo: Arc<R>,
}

impl<R: PolicyRepository> PolicyCommandHandler<R> {
    pub fn new(policy_repo: Arc<R>) -> Self {
        Self { policy_repo }
    }

    /// 创建策略
    pub async fn handle_create(&self, cmd: CreatePolicyCommand) -> AppResult<Policy> {
        // 检查名称是否已存在
        if self.policy_repo.exists_by_name(&cmd.tenant_id, &cmd.name).await? {
            return Err(AppError::conflict(format!(
                "Policy with name '{}' already exists",
                cmd.name
            )));
        }

        let effect: Effect = cmd.effect.parse()
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

        self.policy_repo.create(&policy).await?;

        Ok(policy)
    }

    /// 更新策略
    pub async fn handle_update(&self, cmd: UpdatePolicyCommand) -> AppResult<Policy> {
        let policy_id: PolicyId = cmd.policy_id.parse()
            .map_err(|_| AppError::validation("Invalid policy ID"))?;

        let mut policy = self.policy_repo.find_by_id(&policy_id)
            .await?
            .ok_or_else(|| AppError::not_found("Policy not found"))?;

        let effect: Effect = cmd.effect.parse()
            .map_err(|_| AppError::validation("Invalid effect, must be 'ALLOW' or 'DENY'"))?;

        policy.name = cmd.name;
        policy.description = cmd.description;
        policy.effect = effect;
        policy.subjects = cmd.subjects;
        policy.resources = cmd.resources;
        policy.actions = cmd.actions;
        policy.conditions = cmd.conditions;
        policy.priority = cmd.priority;

        self.policy_repo.update(&policy).await?;

        Ok(policy)
    }

    /// 删除策略
    pub async fn handle_delete(&self, cmd: DeletePolicyCommand) -> AppResult<()> {
        let policy_id: PolicyId = cmd.policy_id.parse()
            .map_err(|_| AppError::validation("Invalid policy ID"))?;

        // 检查策略是否存在
        self.policy_repo.find_by_id(&policy_id)
            .await?
            .ok_or_else(|| AppError::not_found("Policy not found"))?;

        self.policy_repo.delete(&policy_id).await?;

        Ok(())
    }

    /// 激活/停用策略
    pub async fn handle_set_active(&self, cmd: SetPolicyActiveCommand) -> AppResult<Policy> {
        let policy_id: PolicyId = cmd.policy_id.parse()
            .map_err(|_| AppError::validation("Invalid policy ID"))?;

        let mut policy = self.policy_repo.find_by_id(&policy_id)
            .await?
            .ok_or_else(|| AppError::not_found("Policy not found"))?;

        if cmd.is_active {
            policy.activate();
        } else {
            policy.deactivate();
        }

        self.policy_repo.update(&policy).await?;

        Ok(policy)
    }
}
