//! 请求密码重置处理器

use std::str::FromStr;
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

