//! 发送手机验证码处理器

use std::sync::Arc;

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_cqrs_core::CommandHandler;
use cuba_errors::AppResult;
use tracing::{info, warn};
use uuid::Uuid;

use crate::application::commands::user::{
    SendPhoneVerificationCommand, SendPhoneVerificationResult,
};
use crate::domain::services::user::PhoneVerificationService;

/// 发送手机验证码处理器
pub struct SendPhoneVerificationHandler {
    phone_verification_service: Arc<PhoneVerificationService>,
}

impl SendPhoneVerificationHandler {
    pub fn new(phone_verification_service: Arc<PhoneVerificationService>) -> Self {
        Self {
            phone_verification_service,
        }
    }
}

#[async_trait]
impl CommandHandler<SendPhoneVerificationCommand> for SendPhoneVerificationHandler {
    async fn handle(
        &self,
        command: SendPhoneVerificationCommand,
    ) -> AppResult<SendPhoneVerificationResult> {
        info!(
            user_id = %command.user_id,
            tenant_id = %command.tenant_id,
            "Handling SendPhoneVerificationCommand"
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

        // 发送验证码
        match self
            .phone_verification_service
            .send_verification_code(&user_id, &tenant_id)
            .await
        {
            Ok(expires_in_seconds) => {
                info!(
                    user_id = %command.user_id,
                    expires_in_seconds = expires_in_seconds,
                    "Phone verification code sent successfully"
                );

                Ok(SendPhoneVerificationResult {
                    success: true,
                    message: "Verification code sent successfully".to_string(),
                    expires_in_seconds,
                })
            }
            Err(e) => {
                warn!(
                    user_id = %command.user_id,
                    error = %e,
                    "Failed to send phone verification code"
                );

                Ok(SendPhoneVerificationResult {
                    success: false,
                    message: e.to_string(),
                    expires_in_seconds: 0,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::User;
    use crate::domain::services::user::{PhoneVerificationService, SmsSender};
    use crate::domain::value_objects::{Email, Password, Username};
    use crate::infrastructure::persistence::user::{
        PostgresPhoneVerificationRepository, PostgresUserRepository,
    };

    struct MockSmsSender;

    #[async_trait]
    impl SmsSender for MockSmsSender {
        async fn send_verification_code(&self, _phone: &str, _code: &str) -> AppResult<()> {
            Ok(())
        }
    }

    #[sqlx::test]
    async fn test_send_phone_verification_handler(pool: sqlx::PgPool) {
        let phone_verification_repo =
            Arc::new(PostgresPhoneVerificationRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let sms_sender = Arc::new(MockSmsSender);

        let service = Arc::new(PhoneVerificationService::new(
            phone_verification_repo,
            user_repo.clone(),
            sms_sender,
        ));

        let handler = SendPhoneVerificationHandler::new(service);

        let tenant_id = TenantId::from_uuid(Uuid::new_v4());
        let email = Email::new("test@example.com").unwrap();
        let username = Username::new("testuser").unwrap();
        let password = Password::new("Password123!").unwrap();

        // 创建测试用户（带手机号）
        let mut user = User::new(username, email, password, tenant_id.clone()).unwrap();
        user.phone = Some("+8613800138000".to_string());
        user_repo.save(&user, &tenant_id).await.unwrap();

        // 执行命令
        let command = SendPhoneVerificationCommand {
            user_id: user.id.0.to_string(),
            tenant_id: tenant_id.0.to_string(),
        };

        let result = handler.handle(command).await.unwrap();
        assert!(result.success);
        assert!(result.expires_in_seconds > 0);
    }
}
