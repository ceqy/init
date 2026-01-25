//! 用户领域事件

use cuba_event_core::DomainEvent;
use serde::{Deserialize, Serialize};

/// 用户已创建
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreated {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub tenant_id: String,
}

impl DomainEvent for UserCreated {
    fn event_type(&self) -> &'static str {
        "UserCreated"
    }

    fn aggregate_type(&self) -> &'static str {
        "User"
    }

    fn aggregate_id(&self) -> String {
        self.user_id.clone()
    }
}

/// 用户已登录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedIn {
    pub user_id: String,
    pub session_id: String,
    pub ip_address: Option<String>,
    pub device_info: Option<String>,
}

impl DomainEvent for UserLoggedIn {
    fn event_type(&self) -> &'static str {
        "UserLoggedIn"
    }

    fn aggregate_type(&self) -> &'static str {
        "User"
    }

    fn aggregate_id(&self) -> String {
        self.user_id.clone()
    }
}

/// 用户已登出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedOut {
    pub user_id: String,
    pub session_id: String,
}

impl DomainEvent for UserLoggedOut {
    fn event_type(&self) -> &'static str {
        "UserLoggedOut"
    }

    fn aggregate_type(&self) -> &'static str {
        "User"
    }

    fn aggregate_id(&self) -> String {
        self.user_id.clone()
    }
}

/// 密码已修改
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordChanged {
    pub user_id: String,
}

impl DomainEvent for PasswordChanged {
    fn event_type(&self) -> &'static str {
        "PasswordChanged"
    }

    fn aggregate_type(&self) -> &'static str {
        "User"
    }

    fn aggregate_id(&self) -> String {
        self.user_id.clone()
    }
}

/// 2FA 已启用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorEnabled {
    pub user_id: String,
    pub method: String,
}

impl DomainEvent for TwoFactorEnabled {
    fn event_type(&self) -> &'static str {
        "TwoFactorEnabled"
    }

    fn aggregate_type(&self) -> &'static str {
        "User"
    }

    fn aggregate_id(&self) -> String {
        self.user_id.clone()
    }
}

/// 2FA 已禁用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorDisabled {
    pub user_id: String,
}

impl DomainEvent for TwoFactorDisabled {
    fn event_type(&self) -> &'static str {
        "TwoFactorDisabled"
    }

    fn aggregate_type(&self) -> &'static str {
        "User"
    }

    fn aggregate_id(&self) -> String {
        self.user_id.clone()
    }
}
