use crate::infrastructure::events::{EventPublisher, IamDomainEvent};
use async_trait::async_trait;
use cuba_adapter_email::EmailSender;
use std::sync::Arc;

/// 通知监听器
///
/// 监听领域事件并发送相关通知（如邮件）
pub struct NotificationListener {
    email_sender: Arc<dyn EmailSender>,
}

impl NotificationListener {
    pub fn new(email_sender: Arc<dyn EmailSender>) -> Self {
        Self { email_sender }
    }

    async fn handle_user_created(&self, email: &str, username: &str) {
        let subject = "Welcome to Cuba ERP";
        let body = format!(
            "Hello {},\n\nWelcome to Cuba ERP! Your account has been created successfully.",
            username
        );

        if let Err(e) = self
            .email_sender
            .send_html_email(email, subject, &body, None)
            .await
        {
            tracing::error!("Failed to send welcome email: {}", e);
        } else {
            tracing::info!("Welcome email sent to {}", email);
        }
    }

    async fn handle_two_factor_enabled(&self, user_id: &str) {
        // 在实际应用中，这里需要从 repo 获取用户邮箱
        // 为简化演示，这里只记录日志
        tracing::info!("Sending 2FA enabled notification to user {}", user_id);
    }
}

#[async_trait]
impl EventPublisher for NotificationListener {
    async fn publish(&self, event: IamDomainEvent) {
        match event {
            IamDomainEvent::UserCreated {
                email, username, ..
            } => {
                self.handle_user_created(&email, &username).await;
            }
            IamDomainEvent::TwoFactorEnabled { user_id, .. } => {
                self.handle_two_factor_enabled(&user_id.0.to_string()).await;
            }
            _ => {}
        }
    }

    async fn publish_all(&self, events: Vec<IamDomainEvent>) {
        for event in events {
            self.publish(event).await;
        }
    }
}
