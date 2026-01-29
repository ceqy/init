//! 邮件模板系统

use cuba_errors::{AppError, AppResult};
use std::collections::HashMap;
use tera::Tera;
use tracing::debug;

/// 邮件模板管理器
pub struct EmailTemplate {
    tera: Tera,
}

impl EmailTemplate {
    /// 创建新的模板管理器
    pub fn new(template_dir: &str) -> AppResult<Self> {
        let pattern = format!("{}/**/*.html", template_dir);
        let tera = Tera::new(&pattern)
            .map_err(|e| AppError::internal(format!("Failed to load email templates: {}", e)))?;

        debug!(template_dir = %template_dir, "Email templates loaded");

        Ok(Self { tera })
    }

    /// 从内存中的模板字符串创建（用于测试）
    pub fn from_strings(templates: HashMap<String, String>) -> AppResult<Self> {
        let mut tera = Tera::default();

        for (name, content) in templates {
            tera.add_raw_template(&name, &content).map_err(|e| {
                AppError::internal(format!("Failed to add template {}: {}", name, e))
            })?;
        }

        Ok(Self { tera })
    }

    /// 渲染模板
    pub fn render(&self, template_name: &str, context: &serde_json::Value) -> AppResult<String> {
        let context = tera::Context::from_serialize(context)
            .map_err(|e| AppError::internal(format!("Failed to create template context: {}", e)))?;

        self.tera.render(template_name, &context).map_err(|e| {
            AppError::internal(format!(
                "Failed to render template {}: {}",
                template_name, e
            ))
        })
    }

    /// 渲染密码重置邮件
    pub fn render_password_reset(
        &self,
        user_name: &str,
        reset_link: &str,
        expires_in_minutes: u32,
    ) -> AppResult<(String, String)> {
        let mut context = tera::Context::new();
        context.insert("user_name", user_name);
        context.insert("reset_link", reset_link);
        context.insert("expires_in_minutes", &expires_in_minutes);

        // 渲染 HTML 版本
        let html = self
            .tera
            .render("password_reset.html", &context)
            .map_err(|e| AppError::internal(format!("Failed to render HTML template: {}", e)))?;

        // 渲染纯文本版本
        let text = self
            .tera
            .render("password_reset.txt", &context)
            .map_err(|e| AppError::internal(format!("Failed to render text template: {}", e)))?;

        Ok((html, text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_from_strings() {
        let mut templates = HashMap::new();
        templates.insert(
            "test.html".to_string(),
            "<h1>Hello {{ name }}!</h1>".to_string(),
        );

        let template = EmailTemplate::from_strings(templates).unwrap();

        let context = serde_json::json!({
            "name": "World"
        });

        let result = template.render("test.html", &context).unwrap();
        assert_eq!(result, "<h1>Hello World!</h1>");
    }
}
