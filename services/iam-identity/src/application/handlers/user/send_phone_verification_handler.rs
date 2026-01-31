//! 发送手机验证码处理器

use std::sync::Arc;

use async_trait::async_trait;
use common::{TenantId, UserId};
use cqrs_core::CommandHandler;
use errors::AppResult;
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
        let user_id =
            UserId::from_uuid(Uuid::parse_str(&command.user_id).map_err(|e| {
                errors::AppError::validation(format!("Invalid user_id: {}", e))
            })?);
        let tenant_id =
            TenantId::from_uuid(Uuid::parse_str(&command.tenant_id).map_err(|e| {
                errors::AppError::validation(format!("Invalid tenant_id: {}", e))
            })?);

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
