//! 邮件客户端实现

use crate::{EmailConfig, EmailSender, EmailTemplate};
use errors::{AppError, AppResult};
use lettre::message::{MultiPart, SinglePart, header};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use secrecy::ExposeSecret;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

/// 邮件消息
#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: String,
}

/// 邮件客户端
pub struct EmailClient {
    config: EmailConfig,
    template: Option<Arc<EmailTemplate>>,
}

impl EmailClient {
    /// 创建新的邮件客户端
    pub fn new(config: EmailConfig) -> Self {
        Self {
            config,
            template: None,
        }
    }

    /// 设置模板引擎
    pub fn with_template(mut self, template: EmailTemplate) -> Self {
        self.template = Some(Arc::new(template));
        self
    }

    /// 构建 SMTP 传输
    fn build_transport(&self) -> AppResult<SmtpTransport> {
        let credentials = Credentials::new(
            self.config.username.clone(),
            self.config.password.expose_secret().clone(),
        );

        let transport = if self.config.use_tls {
            SmtpTransport::starttls_relay(&self.config.smtp_host)
        } else {
            SmtpTransport::relay(&self.config.smtp_host)
        }
        .map_err(|e| AppError::internal(format!("Failed to create SMTP transport: {}", e)))?
        .port(self.config.smtp_port)
        .credentials(credentials)
        .timeout(Some(Duration::from_secs(self.config.timeout_secs)))
        .build();

        Ok(transport)
    }

    /// 构建邮件消息
    fn build_message(&self, msg: &EmailMessage) -> AppResult<Message> {
        let from = format!("{} <{}>", self.config.from_name, self.config.from_email)
            .parse()
            .map_err(|e| AppError::internal(format!("Invalid from address: {}", e)))?;

        let to = msg
            .to
            .parse()
            .map_err(|e| AppError::validation(format!("Invalid to address: {}", e)))?;

        let message_builder = Message::builder().from(from).to(to).subject(&msg.subject);

        // 构建邮件体
        let body = if let Some(html) = &msg.html_body {
            // HTML + 纯文本备用
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(msg.text_body.clone()),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html.clone()),
                )
        } else {
            // 仅纯文本
            MultiPart::alternative().singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_PLAIN)
                    .body(msg.text_body.clone()),
            )
        };

        let message = message_builder
            .multipart(body)
            .map_err(|e| AppError::internal(format!("Failed to build message: {}", e)))?;

        Ok(message)
    }

    /// 发送邮件
    async fn send_message(&self, message: Message) -> AppResult<()> {
        let transport = self.build_transport()?;

        // 在 tokio 的 blocking 线程池中执行同步操作
        tokio::task::spawn_blocking(move || {
            transport
                .send(&message)
                .map_err(|e| AppError::internal(format!("Failed to send email: {}", e)))
        })
        .await
        .map_err(|e| AppError::internal(format!("Task join error: {}", e)))??;

        Ok(())
    }
}

#[async_trait::async_trait]
impl EmailSender for EmailClient {
    async fn send_text_email(&self, to: &str, subject: &str, body: &str) -> AppResult<()> {
        debug!(to = %to, subject = %subject, "Sending text email");

        let msg = EmailMessage {
            to: to.to_string(),
            subject: subject.to_string(),
            html_body: None,
            text_body: body.to_string(),
        };

        let message = self.build_message(&msg)?;
        self.send_message(message).await?;

        info!(to = %to, subject = %subject, "Text email sent successfully");
        Ok(())
    }

    async fn send_html_email(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
        text_body: Option<&str>,
    ) -> AppResult<()> {
        debug!(to = %to, subject = %subject, "Sending HTML email");

        let msg = EmailMessage {
            to: to.to_string(),
            subject: subject.to_string(),
            html_body: Some(html_body.to_string()),
            text_body: text_body.unwrap_or("").to_string(),
        };

        let message = self.build_message(&msg)?;
        self.send_message(message).await?;

        info!(to = %to, subject = %subject, "HTML email sent successfully");
        Ok(())
    }

    async fn send_template_email(
        &self,
        to: &str,
        subject: &str,
        template_name: &str,
        context: &serde_json::Value,
    ) -> AppResult<()> {
        debug!(to = %to, subject = %subject, template = %template_name, "Sending template email");

        let template = self
            .template
            .as_ref()
            .ok_or_else(|| AppError::internal("Email template not configured"))?;

        let body = template.render(template_name, context)?;

        let msg = EmailMessage {
            to: to.to_string(),
            subject: subject.to_string(),
            html_body: Some(body.clone()),
            text_body: body, // 使用相同内容作为备用
        };

        let message = self.build_message(&msg)?;
        self.send_message(message).await?;

        info!(to = %to, subject = %subject, template = %template_name, "Template email sent successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_message() {
        let config = EmailConfig {
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            username: "user@example.com".to_string(),
            password: secrecy::Secret::new("password".to_string()),
            from_email: "noreply@example.com".to_string(),
            from_name: "Test".to_string(),
            use_tls: true,
            timeout_secs: 30,
        };

        let client = EmailClient::new(config);

        let msg = EmailMessage {
            to: "test@example.com".to_string(),
            subject: "Test Subject".to_string(),
            html_body: Some("<h1>Test</h1>".to_string()),
            text_body: "Test".to_string(),
        };

        let result = client.build_message(&msg);
        assert!(result.is_ok());
    }
}
