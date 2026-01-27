//! 重置密码处理器

use std::sync::Arc;

use async_trait::async_trait;
use cuba_common::TenantId;
use cuba_cqrs_core::CommandHandler;
use cuba_errors::{AppError, AppResult};
use tracing::info;

use crate::application::commands::auth::ResetPasswordCommand;
use crate::domain::services::auth::{PasswordResetService, PasswordService};
use crate::domain::repositories::user::UserRepository;
use crate::domain::value_objects::{Email, Password};

/// 重置密码处理器
pub struct ResetPasswordHandler {
    password_reset_service: Arc<PasswordResetService>,
    password_service: Arc<PasswordService>,
    user_repo: Arc<dyn UserRepository>,
}

impl ResetPasswordHandler {
    pub fn new(
        password_reset_service: Arc<PasswordResetService>,
        password_service: Arc<PasswordService>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            password_reset_service,
            password_service,
            user_repo,
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

        // 3. 验证重置令牌
        let user_id = self
            .password_reset_service
            .verify_reset_token(&command.reset_token, &tenant_id)
            .await?;

        // 4. 查找用户
        let mut user = self
            .user_repo
            .find_by_id(&user_id, &tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        // 5. 验证邮箱匹配
        if user.email != email {
            return Err(AppError::validation("Email does not match"));
        }

        // 6. 验证新密码
        let new_password = Password::new(&command.new_password)
            .map_err(|e| AppError::validation(format!("Invalid password: {}", e)))?;

        // 7. 更新密码
        self.password_service
            .change_password(&mut user, new_password.as_str())?;

        // 8. 保存用户
        self.user_repo.update(&user).await?;

        // 9. 撤销该用户的所有密码重置令牌
        self.password_reset_service
            .revoke_all_tokens(&user_id, &tenant_id)
            .await?;

        info!(
            user_id = %user_id,
            email = %email,
            "Password reset successfully"
        );

        Ok(())
    }
}

