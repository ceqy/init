//! 认证查询

use common::TenantId;
use cqrs_core::Query;

/// 验证令牌查询
#[derive(Debug, Clone)]
pub struct ValidateTokenQuery {
    pub access_token: String,
}

impl Query for ValidateTokenQuery {
    type Result = ValidateTokenResult;
}

/// 验证令牌结果
#[derive(Debug, Clone)]
pub struct ValidateTokenResult {
    pub valid: bool,
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub permissions: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 获取会话查询
#[derive(Debug, Clone)]
pub struct GetSessionByIdQuery {
    pub session_id: String,
    pub tenant_id: TenantId,
}

impl Query for GetSessionByIdQuery {
    type Result = Option<SessionQueryResult>;
}

/// 获取用户活跃会话列表查询
#[derive(Debug, Clone)]
pub struct ListUserSessionsQuery {
    pub user_id: String,
    pub tenant_id: TenantId,
    pub include_expired: bool,
}

impl Query for ListUserSessionsQuery {
    type Result = Vec<SessionQueryResult>;
}

/// 检查密码重置令牌有效性查询
#[derive(Debug, Clone)]
pub struct ValidatePasswordResetTokenQuery {
    pub token: String,
}

impl Query for ValidatePasswordResetTokenQuery {
    type Result = PasswordResetTokenResult;
}

/// 获取用户 2FA 状态查询
#[derive(Debug, Clone)]
pub struct GetUser2FAStatusQuery {
    pub user_id: String,
    pub tenant_id: TenantId,
}

impl Query for GetUser2FAStatusQuery {
    type Result = User2FAStatusResult;
}

/// 获取用户备份码数量查询
#[derive(Debug, Clone)]
pub struct GetBackupCodeCountQuery {
    pub user_id: String,
    pub tenant_id: TenantId,
}

impl Query for GetBackupCodeCountQuery {
    type Result = BackupCodeCountResult;
}

/// 获取用户 WebAuthn 凭证列表查询
#[derive(Debug, Clone)]
pub struct ListWebAuthnCredentialsQuery {
    pub user_id: String,
    pub tenant_id: TenantId,
}

impl Query for ListWebAuthnCredentialsQuery {
    type Result = Vec<WebAuthnCredentialResult>;
}

// ============ 查询结果类型 ============

/// 会话查询结果
#[derive(Debug, Clone)]
pub struct SessionQueryResult {
    pub id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub last_activity_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 密码重置令牌结果
#[derive(Debug, Clone)]
pub struct PasswordResetTokenResult {
    pub valid: bool,
    pub user_id: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

/// 2FA 状态结果
#[derive(Debug, Clone)]
pub struct User2FAStatusResult {
    pub totp_enabled: bool,
    pub webauthn_enabled: bool,
    pub backup_codes_available: bool,
    pub preferred_method: Option<String>,
}

/// 备份码数量结果
#[derive(Debug, Clone)]
pub struct BackupCodeCountResult {
    pub total: u32,
    pub used: u32,
    pub remaining: u32,
}

/// WebAuthn 凭证结果
#[derive(Debug, Clone)]
pub struct WebAuthnCredentialResult {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}
