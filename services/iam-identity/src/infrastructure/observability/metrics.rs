//! IAM Identity Metrics
//!
//! 业务指标记录

use metrics::{counter, gauge, histogram};
use std::time::Instant;

// ============================================================================
// 认证 Metrics
// ============================================================================

/// 记录登录尝试
pub fn record_login_attempt(success: bool, method: &str, has_2fa: bool) {
    let labels = [
        ("success", success.to_string()),
        ("method", method.to_string()),
        ("has_2fa", has_2fa.to_string()),
    ];

    counter!("iam_login_attempts_total", &labels).increment(1);

    if success {
        counter!("iam_login_success_total", &labels).increment(1);
    } else {
        counter!("iam_login_failure_total", &labels).increment(1);
    }
}

/// 记录 2FA 验证
pub fn record_2fa_verification(success: bool, method: &str) {
    let labels = [
        ("success", success.to_string()),
        ("method", method.to_string()),
    ];

    counter!("iam_2fa_verifications_total", &labels).increment(1);

    if success {
        counter!("iam_2fa_success_total", &labels).increment(1);
    } else {
        counter!("iam_2fa_failure_total", &labels).increment(1);
    }
}

/// 记录 2FA 启用
pub fn record_2fa_enabled(method: &str) {
    let labels = [("method", method.to_string())];
    counter!("iam_2fa_enabled_total", &labels).increment(1);
}

/// 记录 2FA 禁用
pub fn record_2fa_disabled(method: &str) {
    let labels = [("method", method.to_string())];
    counter!("iam_2fa_disabled_total", &labels).increment(1);
}

/// 设置当前启用 2FA 的用户数
pub fn set_2fa_enabled_users(count: i64) {
    gauge!("iam_2fa_enabled_users").set(count as f64);
}

/// 记录账户锁定
pub fn record_account_locked(reason: &str) {
    let labels = [("reason", reason.to_string())];
    counter!("iam_account_locked_total", &labels).increment(1);
}

/// 记录账户解锁
pub fn record_account_unlocked() {
    counter!("iam_account_unlocked_total").increment(1);
}

/// 记录可疑登录检测
pub fn record_suspicious_login_detected(reason: &str) {
    let labels = [("reason", reason.to_string())];
    counter!("iam_suspicious_login_detected_total", &labels).increment(1);
}

// ============================================================================
// 会话 Metrics
// ============================================================================

/// 记录会话创建
pub fn record_session_created() {
    counter!("iam_sessions_created_total").increment(1);
}

/// 记录会话撤销
pub fn record_session_revoked(reason: &str) {
    let labels = [("reason", reason.to_string())];
    counter!("iam_sessions_revoked_total", &labels).increment(1);
}

/// 设置当前活跃会话数
pub fn set_active_sessions(count: i64) {
    gauge!("iam_active_sessions").set(count as f64);
}

/// 记录会话过期清理
pub fn record_sessions_expired(count: u64) {
    counter!("iam_sessions_expired_total").increment(count);
}

// ============================================================================
// 密码重置 Metrics
// ============================================================================

/// 记录密码重置请求
pub fn record_password_reset_requested() {
    counter!("iam_password_reset_requested_total").increment(1);
}

/// 记录密码重置完成
pub fn record_password_reset_completed(success: bool) {
    let labels = [("success", success.to_string())];
    counter!("iam_password_reset_completed_total", &labels).increment(1);
}

/// 记录密码重置令牌过期
pub fn record_password_reset_tokens_expired(count: u64) {
    counter!("iam_password_reset_tokens_expired_total").increment(count);
}

// ============================================================================
// WebAuthn Metrics
// ============================================================================

/// 记录 WebAuthn 注册
pub fn record_webauthn_registration(success: bool) {
    let labels = [("success", success.to_string())];
    counter!("iam_webauthn_registrations_total", &labels).increment(1);
}

/// 记录 WebAuthn 认证
pub fn record_webauthn_authentication(success: bool) {
    let labels = [("success", success.to_string())];
    counter!("iam_webauthn_authentications_total", &labels).increment(1);
}

/// 设置 WebAuthn 凭证总数
pub fn set_webauthn_credentials(count: i64) {
    gauge!("iam_webauthn_credentials").set(count as f64);
}

// ============================================================================
// OAuth2 Metrics
// ============================================================================

/// 记录 OAuth2 授权请求
pub fn record_oauth_authorization(grant_type: &str, success: bool) {
    let labels = [
        ("grant_type", grant_type.to_string()),
        ("success", success.to_string()),
    ];
    counter!("iam_oauth_authorizations_total", &labels).increment(1);
}

/// 记录 OAuth2 Token 颁发
pub fn record_oauth_token_issued(token_type: &str) {
    let labels = [("token_type", token_type.to_string())];
    counter!("iam_oauth_tokens_issued_total", &labels).increment(1);
}

/// 记录 OAuth2 Token 撤销
pub fn record_oauth_token_revoked(token_type: &str) {
    let labels = [("token_type", token_type.to_string())];
    counter!("iam_oauth_tokens_revoked_total", &labels).increment(1);
}

/// 设置活跃的 OAuth2 Client 数量
pub fn set_active_oauth_clients(count: i64) {
    gauge!("iam_oauth_active_clients").set(count as f64);
}

// ============================================================================
// 用户 Metrics
// ============================================================================

/// 记录用户注册
pub fn record_user_registered() {
    counter!("iam_users_registered_total").increment(1);
}

/// 记录用户激活
pub fn record_user_activated() {
    counter!("iam_users_activated_total").increment(1);
}

/// 记录用户停用
pub fn record_user_deactivated() {
    counter!("iam_users_deactivated_total").increment(1);
}

/// 设置总用户数
pub fn set_total_users(count: i64) {
    gauge!("iam_total_users").set(count as f64);
}

/// 设置活跃用户数
pub fn set_active_users(count: i64) {
    gauge!("iam_active_users").set(count as f64);
}

// ============================================================================
// 邮箱验证 Metrics
// ============================================================================

/// 记录邮箱验证请求
pub fn record_email_verification_sent() {
    counter!("iam_email_verifications_sent_total").increment(1);
}

/// 记录邮箱验证完成
pub fn record_email_verification_completed(success: bool) {
    let labels = [("success", success.to_string())];
    counter!("iam_email_verifications_completed_total", &labels).increment(1);
}

// ============================================================================
// 租户 Metrics
// ============================================================================

/// 设置租户总数
pub fn set_total_tenants(count: i64) {
    gauge!("iam_total_tenants").set(count as f64);
}

/// 设置活跃租户数
pub fn set_active_tenants(count: i64) {
    gauge!("iam_active_tenants").set(count as f64);
}

/// 记录租户创建
pub fn record_tenant_created() {
    counter!("iam_tenants_created_total").increment(1);
}

// ============================================================================
// API 响应时间 Metrics
// ============================================================================

/// API 请求计时器
pub struct ApiTimer {
    start: Instant,
    service: String,
    method: String,
}

impl ApiTimer {
    pub fn new(service: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            service: service.into(),
            method: method.into(),
        }
    }

    pub fn finish(self, status: &str) {
        let duration = self.start.elapsed().as_secs_f64() * 1000.0;
        let labels = [
            ("service", self.service),
            ("method", self.method),
            ("status", status.to_string()),
        ];

        histogram!("iam_api_request_duration_ms", &labels).record(duration);
        counter!("iam_api_requests_total", &labels).increment(1);
    }
}

// ============================================================================
// 数据库查询时间 Metrics
// ============================================================================

/// 数据库查询计时器
pub struct DbTimer {
    start: Instant,
    operation: String,
    table: String,
}

impl DbTimer {
    pub fn new(operation: impl Into<String>, table: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.into(),
            table: table.into(),
        }
    }

    pub fn finish(self, success: bool) {
        let duration = self.start.elapsed().as_secs_f64() * 1000.0;
        let labels = [
            ("operation", self.operation),
            ("table", self.table),
            ("success", success.to_string()),
        ];

        histogram!("iam_db_query_duration_ms", &labels).record(duration);
        counter!("iam_db_queries_total", &labels).increment(1);
    }
}

// ============================================================================
// 计算 Metrics（定期更新）
// ============================================================================

/// 计算登录成功率
pub fn calculate_login_success_rate(success_count: i64, total_count: i64) -> f64 {
    if total_count == 0 {
        return 0.0;
    }
    (success_count as f64 / total_count as f64) * 100.0
}

/// 计算 2FA 使用率
pub fn calculate_2fa_usage_rate(users_with_2fa: i64, total_users: i64) -> f64 {
    if total_users == 0 {
        return 0.0;
    }
    (users_with_2fa as f64 / total_users as f64) * 100.0
}

/// 设置登录成功率
pub fn set_login_success_rate(rate: f64) {
    gauge!("iam_login_success_rate_percent").set(rate);
}

/// 设置 2FA 使用率
pub fn set_2fa_usage_rate(rate: f64) {
    gauge!("iam_2fa_usage_rate_percent").set(rate);
}
