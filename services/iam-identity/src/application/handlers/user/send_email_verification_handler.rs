//! 发送邮箱验证码处理器

use std::sync::Arc;

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_cqrs_core::CommandHandler;
use cuba_errors::AppResult;
use tracing::{info, warn};
use uuid::Uuid;

use crate::application::commands::user::{
    SendEmailVerificationCommand, SendEmailVerificationResult,
};
use crate::domain::services::user::EmailVerificationService;

/// 发送邮箱验证码处理器
pub struct SendEmailVerificationHandler {
    email_verification_service: Arc<EmailVerificationService>,
}

impl SendEmailVerificationHandler {
    pub fn new(email_verification_service: Arc<EmailVerificationService>) -> Self {
        Self {
            email_verification_service,
        }
    }
}

#[async_trait]
impl CommandHandler<SendEmailVerificationCommand> for SendEmailVerificationHandler {
    async fn handle(
        &self,
        command: SendEmailVerificationCommand,
    ) -> AppResult<SendEmailVerificationResult> {
        info!(
            user_id = %command.user_id,
            tenant_id = %command.tenant_id,
            "Handling SendEmailVerificationCommand"
        );

        // 解析 UUID
        let user_id =
            UserId::from_uuid(Uuid::parse_str(&command.user_id).map_err(|e| {
                cuba_errors::AppError::validation(format!("Invalid user_id: {}", e))
            })?);
        let tenant_id =
            TenantId::from_uuid(Uuid::parse_str(&command.tenant_id).map_err(|e| {
                cuba_errors::AppError::validation(format!("Invalid tenant_id: {}", e))
            })?);

        // 发送验证码
        match self
            .email_verification_service
            .send_verification_code(&user_id, &tenant_id)
            .await
        {
            Ok(expires_in_seconds) => {
                info!(
                    user_id = %command.user_id,
                    expires_in_seconds = expires_in_seconds,
                    "Email verification code sent successfully"
                );

                Ok(SendEmailVerificationResult {
                    success: true,
                    message: "Verification code sent successfully".to_string(),
                    expires_in_seconds,
                })
            }
            Err(e) => {
                warn!(
                    user_id = %command.user_id,
                    error = %e,
                    "Failed to send email verification code"
                );

                Ok(SendEmailVerificationResult {
                    success: false,
                    message: e.to_string(),
                    expires_in_seconds: 0,
                })
            }
        }
    }
}
