//! 验证手机处理器

use std::sync::Arc;

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_cqrs_core::CommandHandler;
use cuba_errors::AppResult;
use tracing::{info, warn};
use uuid::Uuid;

use crate::application::commands::user::{VerifyPhoneCommand, VerifyPhoneResult};
use crate::domain::services::user::PhoneVerificationService;

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

