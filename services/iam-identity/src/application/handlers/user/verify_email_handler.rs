//! 验证邮箱处理器

use std::sync::Arc;

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_cqrs_core::CommandHandler;
use cuba_errors::AppResult;
use tracing::{info, warn};
use uuid::Uuid;

use crate::application::commands::user::{VerifyEmailCommand, VerifyEmailResult};
use crate::domain::services::user::EmailVerificationService;

/// 验证邮箱处理器
pub struct VerifyEmailHandler {
    email_verification_service: Arc<EmailVerificationService>,
}

impl VerifyEmailHandler {
    pub fn new(email_verification_service: Arc<EmailVerificationService>) -> Self {
        Self {
            email_verification_service,
        }
    }
}

#[async_trait]
impl CommandHandler<VerifyEmailCommand> for VerifyEmailHandler {
    async fn handle(&self, command: VerifyEmailCommand) -> AppResult<VerifyEmailResult> {
        info!(
            user_id = %command.user_id,
            tenant_id = %command.tenant_id,
            "Handling VerifyEmailCommand"
        );

        // 解析 UUID
        let user_id = UserId::from_uuid(
            Uuid::parse_str(&command.user_id)
                .map_err(|e| cuba_errors::AppError::validation(format!("Invalid user_id: {}", e)))?,
        );
        let tenant_id = TenantId::from_uuid(
            Uuid::parse_str(&command.tenant_id).map_err(|e| {
                cuba_errors::AppError::validation(format!("Invalid tenant_id: {}", e))
            })?,
        );

        // 验证验证码
        match self
            .email_verification_service
            .verify_code(&user_id, &command.code, &tenant_id)
            .await
        {
            Ok(_) => {
                info!(
                    user_id = %command.user_id,
                    "Email verified successfully"
                );

                Ok(VerifyEmailResult {
                    success: true,
                    message: "Email verified successfully".to_string(),
                })
            }
            Err(e) => {
                warn!(
                    user_id = %command.user_id,
                    error = %e,
                    "Failed to verify email"
                );

                Ok(VerifyEmailResult {
                    success: false,
                    message: e.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::commands::user::SendEmailVerificationCommand;
    use crate::application::handlers::user::SendEmailVerificationHandler;
    use crate::domain::user::User;
    use crate::domain::services::user::EmailVerificationService;
    use crate::domain::value_objects::{Email, Password, Username};
    use crate::infrastructure::persistence::user::{
        PostgresEmailVerificationRepository, PostgresUserRepository,
    };
    use cuba_adapter_email::EmailSender;

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
    async fn test_verify_email_handler(pool: sqlx::PgPool) {
        let email_verification_repo =
            Arc::new(PostgresEmailVerificationRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let email_sender = Arc::new(MockEmailSender);

        let service = Arc::new(EmailVerificationService::new(
            email_verification_repo.clone(),
            user_repo.clone(),
            email_sender,
        ));

        let send_handler = SendEmailVerificationHandler::new(service.clone());
        let verify_handler = VerifyEmailHandler::new(service);

        let tenant_id = TenantId::from_uuid(Uuid::new_v4());
        let email = Email::new("test@example.com").unwrap();
        let username = Username::new("testuser").unwrap();
        let password = Password::new("Password123!").unwrap();

        // 创建测试用户
        let user = User::new(username, email, password, tenant_id.clone()).unwrap();
        user_repo.save(&user, &tenant_id).await.unwrap();

        // 发送验证码
        let send_command = SendEmailVerificationCommand {
            user_id: user.id.0.to_string(),
            tenant_id: tenant_id.0.to_string(),
        };
        send_handler.handle(send_command).await.unwrap();

        // 获取验证码
        let verification = email_verification_repo
            .find_latest_by_user_id(&user.id, &tenant_id)
            .await
            .unwrap()
            .unwrap();

        // 验证验证码
        let verify_command = VerifyEmailCommand {
            user_id: user.id.0.to_string(),
            code: verification.code,
            tenant_id: tenant_id.0.to_string(),
        };

        let result = verify_handler.handle(verify_command).await.unwrap();
        assert!(result.success);
    }
}
