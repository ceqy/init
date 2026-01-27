//! 验证手机处理器

use std::sync::Arc;

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_cqrs_core::CommandHandler;
use cuba_errors::AppResult;
use tracing::{info, warn};
use uuid::Uuid;

use crate::shared::application::commands::{VerifyPhoneCommand, VerifyPhoneResult};
use crate::shared::domain::services::PhoneVerificationService;

/// 验证手机处理器
pub struct VerifyPhoneHandler {
    phone_verification_service: Arc<PhoneVerificationService>,
}

impl VerifyPhoneHandler {
    pub fn new(phone_verification_service: Arc<PhoneVerificationService>) -> Self {
        Self {
            phone_verification_service,
        }
    }
}

#[async_trait]
impl CommandHandler<VerifyPhoneCommand> for VerifyPhoneHandler {
    async fn handle(&self, command: VerifyPhoneCommand) -> AppResult<VerifyPhoneResult> {
        info!(
            user_id = %command.user_id,
            tenant_id = %command.tenant_id,
            "Handling VerifyPhoneCommand"
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
            .phone_verification_service
            .verify_code(&user_id, &command.code, &tenant_id)
            .await
        {
            Ok(_) => {
                info!(
                    user_id = %command.user_id,
                    "Phone verified successfully"
                );

                Ok(VerifyPhoneResult {
                    success: true,
                    message: "Phone verified successfully".to_string(),
                })
            }
            Err(e) => {
                warn!(
                    user_id = %command.user_id,
                    error = %e,
                    "Failed to verify phone"
                );

                Ok(VerifyPhoneResult {
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
    use crate::shared::application::commands::SendPhoneVerificationCommand;
    use crate::shared::application::handlers::SendPhoneVerificationHandler;
    use crate::shared::domain::entities::User;
    use crate::shared::domain::services::{PhoneVerificationService, SmsSender};
    use crate::shared::domain::value_objects::{Email, Password, Username};
    use crate::shared::infrastructure::persistence::{
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
    async fn test_verify_phone_handler(pool: sqlx::PgPool) {
        let phone_verification_repo =
            Arc::new(PostgresPhoneVerificationRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let sms_sender = Arc::new(MockSmsSender);

        let service = Arc::new(PhoneVerificationService::new(
            phone_verification_repo.clone(),
            user_repo.clone(),
            sms_sender,
        ));

        let send_handler = SendPhoneVerificationHandler::new(service.clone());
        let verify_handler = VerifyPhoneHandler::new(service);

        let tenant_id = TenantId::from_uuid(Uuid::new_v4());
        let email = Email::new("test@example.com").unwrap();
        let username = Username::new("testuser").unwrap();
        let password = Password::new("Password123!").unwrap();

        // 创建测试用户（带手机号）
        let mut user = User::new(username, email, password, tenant_id.clone()).unwrap();
        user.phone = Some("+8613800138000".to_string());
        user_repo.save(&user, &tenant_id).await.unwrap();

        // 发送验证码
        let send_command = SendPhoneVerificationCommand {
            user_id: user.id.0.to_string(),
            tenant_id: tenant_id.0.to_string(),
        };
        send_handler.handle(send_command).await.unwrap();

        // 获取验证码
        let verification = phone_verification_repo
            .find_latest_by_user_id(&user.id, &tenant_id)
            .await
            .unwrap()
            .unwrap();

        // 验证验证码
        let verify_command = VerifyPhoneCommand {
            user_id: user.id.0.to_string(),
            code: verification.code,
            tenant_id: tenant_id.0.to_string(),
        };

        let result = verify_handler.handle(verify_command).await.unwrap();
        assert!(result.success);
    }
}
