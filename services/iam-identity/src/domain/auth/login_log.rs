//! 登录日志实体

use chrono::{DateTime, Utc};
use cuba_common::{TenantId, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 登录日志 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LoginLogId(pub Uuid);

impl LoginLogId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for LoginLogId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for LoginLogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 登录结果
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginResult {
    Success,
    Failed,
}

/// 登录失败原因
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginFailureReason {
    InvalidCredentials,
    AccountLocked,
    AccountDisabled,
    AccountInactive,
    TwoFactorRequired,
    TwoFactorFailed,
    CaptchaRequired,
    CaptchaFailed,
    IpBlocked,
    TenantInactive,
    Other(String),
}

impl std::fmt::Display for LoginFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "Invalid credentials"),
            Self::AccountLocked => write!(f, "Account locked"),
            Self::AccountDisabled => write!(f, "Account disabled"),
            Self::AccountInactive => write!(f, "Account inactive"),
            Self::TwoFactorRequired => write!(f, "Two-factor authentication required"),
            Self::TwoFactorFailed => write!(f, "Two-factor authentication failed"),
            Self::CaptchaRequired => write!(f, "Captcha required"),
            Self::CaptchaFailed => write!(f, "Captcha failed"),
            Self::IpBlocked => write!(f, "IP address blocked"),
            Self::TenantInactive => write!(f, "Tenant inactive"),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// 设备类型（Desktop, Mobile, Tablet）
    pub device_type: String,
    /// 操作系统
    pub os: String,
    /// 操作系统版本
    pub os_version: Option<String>,
    /// 浏览器
    pub browser: String,
    /// 浏览器版本
    pub browser_version: Option<String>,
    /// 是否为移动设备
    pub is_mobile: bool,
    /// 设备指纹
    pub device_fingerprint: Option<String>,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            device_type: "Unknown".to_string(),
            os: "Unknown".to_string(),
            os_version: None,
            browser: "Unknown".to_string(),
            browser_version: None,
            is_mobile: false,
            device_fingerprint: None,
        }
    }
}

impl DeviceInfo {
    /// 从 User-Agent 解析设备信息
    pub fn from_user_agent(user_agent: &str) -> Self {
        // 简化的解析逻辑，实际应使用专门的 User-Agent 解析库
        let is_mobile = user_agent.contains("Mobile") || user_agent.contains("Android");
        
        let device_type = if user_agent.contains("Tablet") {
            "Tablet"
        } else if is_mobile {
            "Mobile"
        } else {
            "Desktop"
        }.to_string();

        let (os, os_version) = Self::parse_os(user_agent);
        let (browser, browser_version) = Self::parse_browser(user_agent);

        Self {
            device_type,
            os,
            os_version,
            browser,
            browser_version,
            is_mobile,
            device_fingerprint: None,
        }
    }

    fn parse_os(user_agent: &str) -> (String, Option<String>) {
        if user_agent.contains("Windows") {
            ("Windows".to_string(), None)
        } else if user_agent.contains("Mac OS") {
            ("macOS".to_string(), None)
        } else if user_agent.contains("Linux") {
            ("Linux".to_string(), None)
        } else if user_agent.contains("Android") {
            ("Android".to_string(), None)
        } else if user_agent.contains("iOS") || user_agent.contains("iPhone") {
            ("iOS".to_string(), None)
        } else {
            ("Unknown".to_string(), None)
        }
    }

    fn parse_browser(user_agent: &str) -> (String, Option<String>) {
        if user_agent.contains("Chrome") {
            ("Chrome".to_string(), None)
        } else if user_agent.contains("Firefox") {
            ("Firefox".to_string(), None)
        } else if user_agent.contains("Safari") {
            ("Safari".to_string(), None)
        } else if user_agent.contains("Edge") {
            ("Edge".to_string(), None)
        } else {
            ("Unknown".to_string(), None)
        }
    }

    /// 生成设备指纹
    pub fn fingerprint(&self) -> String {
        format!(
            "{}:{}:{}",
            self.device_type,
            self.os,
            self.browser
        )
    }
}

/// 登录日志实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginLog {
    /// 日志 ID
    pub id: LoginLogId,
    /// 用户 ID（可能为空，如果用户名不存在）
    pub user_id: Option<UserId>,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 用户名
    pub username: String,
    /// IP 地址
    pub ip_address: String,
    /// User-Agent
    pub user_agent: String,
    /// 设备信息
    pub device_info: DeviceInfo,
    /// 登录结果
    pub result: LoginResult,
    /// 失败原因
    pub failure_reason: Option<LoginFailureReason>,
    /// 地理位置（国家）
    pub country: Option<String>,
    /// 地理位置（城市）
    pub city: Option<String>,
    /// 是否为可疑登录
    pub is_suspicious: bool,
    /// 可疑原因
    pub suspicious_reason: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl LoginLog {
    /// 创建成功的登录日志
    pub fn success(
        user_id: UserId,
        tenant_id: TenantId,
        username: String,
        ip_address: String,
        user_agent: String,
    ) -> Self {
        let device_info = DeviceInfo::from_user_agent(&user_agent);

        Self {
            id: LoginLogId::new(),
            user_id: Some(user_id),
            tenant_id,
            username,
            ip_address,
            user_agent,
            device_info,
            result: LoginResult::Success,
            failure_reason: None,
            country: None,
            city: None,
            is_suspicious: false,
            suspicious_reason: None,
            created_at: Utc::now(),
        }
    }

    /// 创建失败的登录日志
    pub fn failure(
        user_id: Option<UserId>,
        tenant_id: TenantId,
        username: String,
        ip_address: String,
        user_agent: String,
        reason: LoginFailureReason,
    ) -> Self {
        let device_info = DeviceInfo::from_user_agent(&user_agent);

        Self {
            id: LoginLogId::new(),
            user_id,
            tenant_id,
            username,
            ip_address,
            user_agent,
            device_info,
            result: LoginResult::Failed,
            failure_reason: Some(reason),
            country: None,
            city: None,
            is_suspicious: false,
            suspicious_reason: None,
            created_at: Utc::now(),
        }
    }

    /// 标记为可疑登录
    pub fn mark_suspicious(&mut self, reason: String) {
        self.is_suspicious = true;
        self.suspicious_reason = Some(reason);
    }

    /// 设置地理位置
    pub fn set_location(&mut self, country: String, city: String) {
        self.country = Some(country);
        self.city = Some(city);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_from_user_agent() {
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        let device_info = DeviceInfo::from_user_agent(user_agent);

        assert_eq!(device_info.device_type, "Desktop");
        assert_eq!(device_info.os, "Windows");
        assert_eq!(device_info.browser, "Chrome");
        assert!(!device_info.is_mobile);
    }

    #[test]
    fn test_device_info_mobile() {
        let user_agent = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1";
        let device_info = DeviceInfo::from_user_agent(user_agent);

        assert_eq!(device_info.device_type, "Mobile");
        assert_eq!(device_info.os, "iOS");
        assert!(device_info.is_mobile);
    }

    #[test]
    fn test_login_log_success() {
        let user_id = UserId::new();
        let tenant_id = TenantId::new();
        let log = LoginLog::success(
            user_id.clone(),
            tenant_id,
            "testuser".to_string(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
        );

        assert_eq!(log.result, LoginResult::Success);
        assert_eq!(log.user_id, Some(user_id));
        assert!(log.failure_reason.is_none());
    }

    #[test]
    fn test_login_log_failure() {
        let tenant_id = TenantId::new();
        let log = LoginLog::failure(
            None,
            tenant_id,
            "testuser".to_string(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            LoginFailureReason::InvalidCredentials,
        );

        assert_eq!(log.result, LoginResult::Failed);
        assert!(log.user_id.is_none());
        assert_eq!(log.failure_reason, Some(LoginFailureReason::InvalidCredentials));
    }

    #[test]
    fn test_mark_suspicious() {
        let user_id = UserId::new();
        let tenant_id = TenantId::new();
        let mut log = LoginLog::success(
            user_id,
            tenant_id,
            "testuser".to_string(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
        );

        assert!(!log.is_suspicious);

        log.mark_suspicious("Login from unusual location".to_string());
        assert!(log.is_suspicious);
        assert_eq!(log.suspicious_reason, Some("Login from unusual location".to_string()));
    }
}
