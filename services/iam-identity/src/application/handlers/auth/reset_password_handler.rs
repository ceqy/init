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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::persistence::auth::PostgresPasswordResetRepository;
    use crate::domain::user::User;
    use crate::domain::value_objects::Username;
    use crate::infrastructure::persistence::user::PostgresUserRepository;
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_reset_password(pool: sqlx::PgPool) {
        let password_reset_repo = Arc::new(PostgresPasswordResetRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let password_reset_service = Arc::new(PasswordResetService::new(
            password_reset_repo,
            user_repo.clone(),
        ));
        let password_service = Arc::new(PasswordService::new());

        let handler = ResetPasswordHandler::new(
            password_reset_service.clone(),
            password_service,
            user_repo.clone(),
        );

        let tenant_id = TenantId::from_uuid(Uuid::new_v4());
        let email = Email::new("test@example.com").unwrap();
        let username = Username::new("testuser").unwrap();
        let old_password = Password::new("OldPassword123!").unwrap();

        // 创建测试用户
        let user = User::new(
            username,
            email.clone(),
            old_password.clone(),
            tenant_id.clone(),
        )
        .unwrap();
        user_repo.save(&user, &tenant_id).await.unwrap();

        // 生成重置令牌
        let (token, _) = password_reset_service
            .generate_reset_token(&email, &tenant_id, 15)
            .await
            .unwrap();

        // 执行重置密码命令
        let command = ResetPasswordCommand {
            email: email.to_string(),
            reset_token: token.clone(),
            new_password: "NewPassword456!".to_string(),
            tenant_id: tenant_id.to_string(),
        };

        let result = handler.handle(command).await;
        assert!(result.is_ok());

        // 验证密码已更改
        let updated_user = user_repo
            .find_by_id(&user.id, &tenant_id)
            .await
            .unwrap()
            .unwrap();
        assert_ne!(updated_user.password_hash, old_password.hash());

        // 验证令牌已被撤销（再次使用应该失败）
        let verify_result = password_reset_service
            .verify_reset_token(&token, &tenant_id)
            .await;
        assert!(verify_result.is_err());
    }

    #[sqlx::test]
    async fn test_reset_password_with_invalid_token(pool: sqlx::PgPool) {
        let password_reset_repo = Arc::new(PostgresPasswordResetRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let password_reset_service = Arc::new(PasswordResetService::new(
            password_reset_repo,
            user_repo.clone(),
        ));
        let password_service = Arc::new(PasswordService::new());

        let handler = ResetPasswordHandler::new(
            password_reset_service,
            password_service,
            user_repo,
        );

        let tenant_id = TenantId::from_uuid(Uuid::new_v4());

        // 执行重置密码命令（无效令牌）
        let command = ResetPasswordCommand {
            email: "test@example.com".to_string(),
            reset_token: "invalid_token".to_string(),
            new_password: "NewPassword456!".to_string(),
            tenant_id: tenant_id.to_string(),
        };

        let result = handler.handle(command).await;
        assert!(result.is_err());
    }
}
