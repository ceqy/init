//! 租户上下文值对象

use cuba_common::TenantId;
use serde::{Deserialize, Serialize};

/// 租户上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 租户名称
    pub tenant_name: String,
    /// 租户设置
    pub settings: TenantSettings,
}

impl TenantContext {
    pub fn new(tenant_id: TenantId, tenant_name: String, settings: TenantSettings) -> Self {
        Self {
            tenant_id,
            tenant_name,
            settings,
        }
    }

    /// 验证租户是否激活
    pub fn is_active(&self) -> bool {
        self.settings.is_active
    }

    /// 检查是否需要 2FA
    pub fn requires_2fa(&self) -> bool {
        self.settings.password_policy.require_2fa
    }

    /// 检查用户数量是否超限
    pub fn is_user_limit_reached(&self, current_users: i64) -> bool {
        if let Some(max_users) = self.settings.max_users {
            current_users >= max_users
        } else {
            false
        }
    }

    /// 检查 OAuth scope 是否允许
    pub fn is_oauth_scope_allowed(&self, scope: &str) -> bool {
        self.settings.allowed_oauth_scopes.contains(&scope.to_string())
    }
}

/// 租户设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSettings {
    /// 是否激活
    pub is_active: bool,
    /// 密码策略
    pub password_policy: PasswordPolicy,
    /// 最大用户数（None 表示无限制）
    pub max_users: Option<i64>,
    /// 允许的 OAuth scopes
    pub allowed_oauth_scopes: Vec<String>,
    /// 会话超时时间（秒）
    pub session_timeout_seconds: i64,
    /// 是否允许多设备登录
    pub allow_multiple_sessions: bool,
}

impl Default for TenantSettings {
    fn default() -> Self {
        Self {
            is_active: true,
            password_policy: PasswordPolicy::default(),
            max_users: None,
            allowed_oauth_scopes: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
            ],
            session_timeout_seconds: 3600,
            allow_multiple_sessions: true,
        }
    }
}

/// 密码策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// 最小长度
    pub min_length: usize,
    /// 是否需要大写字母
    pub require_uppercase: bool,
    /// 是否需要小写字母
    pub require_lowercase: bool,
    /// 是否需要数字
    pub require_digit: bool,
    /// 是否需要特殊字符
    pub require_special_char: bool,
    /// 是否需要 2FA
    pub require_2fa: bool,
    /// 密码过期天数（None 表示不过期）
    pub password_expiry_days: Option<i64>,
    /// 密码历史记录数量（防止重复使用）
    pub password_history_count: usize,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special_char: true,
            require_2fa: false,
            password_expiry_days: None,
            password_history_count: 5,
        }
    }
}

impl PasswordPolicy {
    /// 验证密码是否符合策略
    pub fn validate(&self, password: &str) -> Result<(), PasswordPolicyError> {
        if password.len() < self.min_length {
            return Err(PasswordPolicyError::TooShort {
                min: self.min_length,
                actual: password.len(),
            });
        }

        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(PasswordPolicyError::MissingUppercase);
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(PasswordPolicyError::MissingLowercase);
        }

        if self.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(PasswordPolicyError::MissingDigit);
        }

        if self.require_special_char
            && !password
                .chars()
                .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
        {
            return Err(PasswordPolicyError::MissingSpecialChar);
        }

        Ok(())
    }
}

/// 密码策略错误
#[derive(Debug, thiserror::Error)]
pub enum PasswordPolicyError {
    #[error("Password is too short: minimum {min} characters, got {actual}")]
    TooShort { min: usize, actual: usize },

    #[error("Password must contain at least one uppercase letter")]
    MissingUppercase,

    #[error("Password must contain at least one lowercase letter")]
    MissingLowercase,

    #[error("Password must contain at least one digit")]
    MissingDigit,

    #[error("Password must contain at least one special character")]
    MissingSpecialChar,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_policy_validation() {
        let policy = PasswordPolicy::default();

        // 有效密码
        assert!(policy.validate("Password123!").is_ok());

        // 太短
        assert!(policy.validate("Pass1!").is_err());

        // 缺少大写字母
        assert!(policy.validate("password123!").is_err());

        // 缺少小写字母
        assert!(policy.validate("PASSWORD123!").is_err());

        // 缺少数字
        assert!(policy.validate("Password!").is_err());

        // 缺少特殊字符
        assert!(policy.validate("Password123").is_err());
    }

    #[test]
    fn test_tenant_context_user_limit() {
        let mut settings = TenantSettings::default();
        settings.max_users = Some(10);

        let context = TenantContext::new(
            TenantId::new(),
            "Test Tenant".to_string(),
            settings,
        );

        assert!(!context.is_user_limit_reached(5));
        assert!(!context.is_user_limit_reached(9));
        assert!(context.is_user_limit_reached(10));
        assert!(context.is_user_limit_reached(15));
    }

    #[test]
    fn test_oauth_scope_allowed() {
        let settings = TenantSettings::default();
        let context = TenantContext::new(
            TenantId::new(),
            "Test Tenant".to_string(),
            settings,
        );

        assert!(context.is_oauth_scope_allowed("openid"));
        assert!(context.is_oauth_scope_allowed("profile"));
        assert!(!context.is_oauth_scope_allowed("admin"));
    }
}
