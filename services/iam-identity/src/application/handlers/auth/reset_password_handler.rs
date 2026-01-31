//! 重置密码处理器

use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use common::TenantId;
use cqrs_core::CommandHandler;
use errors::{AppError, AppResult};
use tracing::info;

use crate::application::commands::auth::ResetPasswordCommand;
use crate::domain::services::auth::{PasswordResetService, PasswordService};
use crate::domain::unit_of_work::UnitOfWorkFactory;
use crate::domain::value_objects::{Email, Password};
use crate::infrastructure::events::{EventPublisher, IamDomainEvent};

/// 重置密码处理器
pub struct ResetPasswordHandler {
    password_reset_service: Arc<PasswordResetService>,
    password_service: Arc<PasswordService>,
    uow_factory: Arc<dyn UnitOfWorkFactory>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl ResetPasswordHandler {
    pub fn new(
        password_reset_service: Arc<PasswordResetService>,
        password_service: Arc<PasswordService>,
        uow_factory: Arc<dyn UnitOfWorkFactory>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            password_reset_service,
            password_service,
            uow_factory,
            event_publisher,
        }
    }
}

#[async_trait]
impl CommandHandler<ResetPasswordCommand> for ResetPasswordHandler {
    async fn handle(&self, command: ResetPasswordCommand) -> AppResult<()> {
        info!(email = %command.email, tenant_id = %command.tenant_id, "Handling ResetPassword command");

        // 1. 验证邮箱格式
        let email = Email::new(&command.email)
            .map_err(|e| AppError::validation(format!("Invalid email: {}", e)))?;

        // 2. 解析租户 ID
        let tenant_id = TenantId::from_str(&command.tenant_id)
            .map_err(|_| AppError::validation("Invalid tenant ID"))?;

        // 3. 验证重置令牌（这会标记令牌为已使用）
        let user_id = self
            .password_reset_service
            .verify_reset_token(&command.reset_token, &tenant_id)
            .await?;

        // 4. 验证新密码格式
        let new_password = Password::new(&command.new_password)
            .map_err(|e| AppError::validation(format!("Invalid password: {}", e)))?;

        // 5. 开始事务 - 保证以下操作的原子性
        let uow = self.uow_factory.begin().await?;

        // 6. 查找用户
        let mut user = uow
            .users()
            .find_by_id(&user_id, &tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        // 7. 验证邮箱匹配
        if user.email != email {
            return Err(AppError::validation("Email does not match"));
        }

        // 8. 更新密码
        self.password_service
            .change_password(&mut user, new_password.as_str())?;

        // 9. 保存用户
        uow.users().update(&user).await?;

        // 10. 撤销该用户的所有密码重置令牌
        uow.password_resets()
            .delete_by_user_id(&user_id, &tenant_id)
            .await?;

        // 11. 撤销该用户的所有会话（强制重新登录）
        uow.sessions()
            .revoke_all_by_user_id(&user_id, &tenant_id)
            .await?;

        // 12. 提交事务
        uow.commit().await?;

        // 13. 发布密码修改事件（事务提交后）
        let event = IamDomainEvent::PasswordChanged {
            user_id: user_id.clone(),
            tenant_id: tenant_id.clone(),
            timestamp: Utc::now(),
        };
        self.event_publisher.publish(event).await;

        info!(
            user_id = %user_id,
            email = %email,
            "Password reset successfully"
        );

        Ok(())
    }
}
