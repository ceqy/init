//! 请求密码重置处理器

use std::sync::Arc;

use async_trait::async_trait;
use cuba_adapter_email::EmailSender;
use cuba_common::TenantId;
use cuba_cqrs_core::CommandHandler;
use cuba_errors::{AppError, AppResult};
use tracing::{info, warn};

use crate::application::commands::auth::RequestPasswordResetCommand;
use crate::domain::services::auth::PasswordResetService;
use crate::domain::value_objects::Email;

/// 请求密码重置处理器
pub struct RequestPasswordResetHandler {
    password_reset_service: Arc<PasswordResetService>,
    email_sender: Arc<dyn EmailSender>,
    expires_in_minutes: i64,
}

impl RequestPasswordResetHandler {
    pub fn new(
        password_reset_service: Arc<PasswordResetService>,
        email_sender: Arc<dyn EmailSender>,
        expires_in_minutes: i64,
    ) -> Self {
        Self {
            password_reset_service,
            email_sender,
            expires_in_minutes,
        }
    }
}

#[async_trait]
impl CommandHandler<RequestPasswordResetCommand> for RequestPasswordResetHandler {
    async fn handle(&self, command: RequestPasswordResetCommand) -> AppResult<()> {
        info!(email = %command.email, tenant_id = %command.tenant_id, "Handling RequestPasswordReset command");

        // 1. 验证邮箱格式
        let email = Email::new(&command.email)
            .map_err(|e| AppError::validation(format!("Invalid email: {}", e)))?;

        // 2. 解析租户 ID
        let tenant_id = TenantId::from_str(&command.tenant_id)
            .map_err(|_| AppError::validation("Invalid tenant ID"))?;

        // 3. 生成重置令牌
        let result = self
            .password_reset_service
            .generate_reset_token(&email, &tenant_id, self.expires_in_minutes)
            .await;

        // 4. 处理结果（为了安全，即使用户不存在也返回成功）
        match result {
            Ok((token, user_id)) => {
                // 构建重置链接
                let reset_link = format!("{}?token={}", command.reset_url_base, token);

                // 发送邮件
                let subject = "密码重置请求";
                let body = format!(
                    r#"您好！

我们收到了您的密码重置请求。请点击以下链接重置您的密码：

{}

此链接将在 {} 分钟后失效。

如果您没有请求重置密码，请忽略此邮件。

---
Cuba ERP 系统
"#,
                    reset_link, self.expires_in_minutes
                );

                self.email_sender
                    .send_text_email(&command.email, subject, &body)
                    .await?;

                info!(
                    user_id = %user_id,
                    email = %command.email,
                    "Password reset email sent successfully"
                );
            }
            Err(e) => {
                // 记录错误但不暴露给用户（防止用户枚举）
                warn!(
                    email = %command.email,
                    error = %e,
                    "Failed to generate password reset token"
                );

                // 为了安全，仍然返回成功
                info!(
                    email = %command.email,
                    "Password reset request processed (user may not exist)"
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::auth::PasswordResetRepository;
    use crate::infrastructure::persistence::auth::PostgresPasswordResetRepository;
    use crate::domain::user::User;
    use crate::domain::repositories::user::UserRepository;
    use crate::domain::value_objects::{Password, Username};
    use crate::infrastructure::persistence::user::PostgresUserRepository;
    use uuid::Uuid;

    // Mock EmailSender for testing
    struct MockEmailSender;

    #[async_trait]
    impl EmailSender for MockEmailSender {
        async fn send_text_email(&self, _to: &str, _subject: &str, _body: &str) -> AppResult<()> {
            Ok(())
        }

        async fn send_html_email(
            &self,
            _to: &str,
            _subject: &str,
            _html_body: &str,
            _text_body: Option<&str>,
        ) -> AppResult<()> {
            Ok(())
        }

        async fn send_template_email(
            &self,
            _to: &str,
            _subject: &str,
            _template_name: &str,
            _context: &serde_json::Value,
        ) -> AppResult<()> {
            Ok(())
        }
    }

    #[sqlx::test]
    async fn test_request_password_reset(pool: sqlx::PgPool) {
        let password_reset_repo = Arc::new(PostgresPasswordResetRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let password_reset_service = Arc::new(PasswordResetService::new(
            password_reset_repo,
            user_repo.clone(),
        ));
        let email_sender = Arc::new(MockEmailSender);

        let handler = RequestPasswordResetHandler::new(password_reset_service, email_sender, 15);

        let tenant_id = TenantId::from_uuid(Uuid::new_v4());
        let email = Email::new("test@example.com").unwrap();
        let username = Username::new("testuser").unwrap();
        let password = Password::new("Password123!").unwrap();

        // 创建测试用户
        let user = User::new(username, email.clone(), password, tenant_id.clone()).unwrap();
        user_repo.save(&user, &tenant_id).await.unwrap();

        // 执行命令
        let command = RequestPasswordResetCommand {
            email: email.to_string(),
            tenant_id: tenant_id.to_string(),
            reset_url_base: "https://app.example.com/reset-password".to_string(),
        };

        let result = handler.handle(command).await;
        assert!(result.is_ok());
    }

    #[sqlx::test]
    async fn test_request_password_reset_nonexistent_user(pool: sqlx::PgPool) {
        let password_reset_repo = Arc::new(PostgresPasswordResetRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let password_reset_service = Arc::new(PasswordResetService::new(
            password_reset_repo,
            user_repo,
        ));
        let email_sender = Arc::new(MockEmailSender);

        let handler = RequestPasswordResetHandler::new(password_reset_service, email_sender, 15);

        let tenant_id = TenantId::from_uuid(Uuid::new_v4());

        // 执行命令（用户不存在）
        let command = RequestPasswordResetCommand {
            email: "nonexistent@example.com".to_string(),
            tenant_id: tenant_id.to_string(),
            reset_url_base: "https://app.example.com/reset-password".to_string(),
        };

        // 应该返回成功（不暴露用户是否存在）
        let result = handler.handle(command).await;
        assert!(result.is_ok());
    }
}
