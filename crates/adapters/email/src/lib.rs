//! Email 适配器
//!
//! 提供邮件发送功能，支持：
//! - SMTP 邮件发送
//! - 模板渲染
//! - HTML 和纯文本邮件

mod client;
mod template;

pub use client::{EmailClient, EmailMessage};
pub use template::EmailTemplate;

// 重新导出 EmailConfig
pub use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    #[serde(default)]
    pub use_tls: bool,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_timeout_secs() -> u64 {
    30
}

use cuba_errors::AppResult;

/// 邮件发送接口
#[async_trait::async_trait]
pub trait EmailSender: Send + Sync {
    /// 发送纯文本邮件
    async fn send_text_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> AppResult<()>;

    /// 发送 HTML 邮件
    async fn send_html_email(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
        text_body: Option<&str>,
    ) -> AppResult<()>;

    /// 发送模板邮件
    async fn send_template_email(
        &self,
        to: &str,
        subject: &str,
        template_name: &str,
        context: &serde_json::Value,
    ) -> AppResult<()>;
}
