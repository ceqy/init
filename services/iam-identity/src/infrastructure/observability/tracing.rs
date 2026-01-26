//! 分布式追踪
//!
//! 使用 tracing 和 OpenTelemetry 进行分布式追踪

use tracing::{info, warn, error, debug, Span};
use tracing::field::Empty;

// ============================================================================
// Span 辅助函数
// ============================================================================

/// 创建认证 Span
pub fn auth_span(operation: &str) -> Span {
    tracing::info_span!(
        "auth",
        operation = operation,
        user_id = Empty,
        tenant_id = Empty,
        success = Empty,
        error = Empty,
    )
}

/// 创建用户操作 Span
pub fn user_span(operation: &str) -> Span {
    tracing::info_span!(
        "user",
        operation = operation,
        user_id = Empty,
        tenant_id = Empty,
        success = Empty,
    )
}

/// 创建会话操作 Span
pub fn session_span(operation: &str) -> Span {
    tracing::info_span!(
        "session",
        operation = operation,
        session_id = Empty,
        user_id = Empty,
        tenant_id = Empty,
    )
}

/// 创建 OAuth2 操作 Span
pub fn oauth_span(operation: &str) -> Span {
    tracing::info_span!(
        "oauth",
        operation = operation,
        client_id = Empty,
        grant_type = Empty,
        scope = Empty,
        success = Empty,
    )
}

/// 创建数据库操作 Span
pub fn db_span(operation: &str, table: &str) -> Span {
    tracing::info_span!(
        "db",
        operation = operation,
        table = table,
        duration_ms = Empty,
        success = Empty,
    )
}

/// 创建缓存操作 Span
pub fn cache_span(operation: &str) -> Span {
    tracing::info_span!(
        "cache",
        operation = operation,
        key = Empty,
        hit = Empty,
    )
}

// ============================================================================
// 结构化日志辅助函数
// ============================================================================

/// 记录认证成功
pub fn log_auth_success(user_id: &str, tenant_id: &str, method: &str) {
    info!(
        user_id = user_id,
        tenant_id = tenant_id,
        method = method,
        "Authentication successful"
    );
}

/// 记录认证失败
pub fn log_auth_failure(username: &str, tenant_id: &str, reason: &str) {
    warn!(
        username = username,
        tenant_id = tenant_id,
        reason = reason,
        "Authentication failed"
    );
}

/// 记录账户锁定
pub fn log_account_locked(user_id: &str, tenant_id: &str, reason: &str) {
    warn!(
        user_id = user_id,
        tenant_id = tenant_id,
        reason = reason,
        "Account locked"
    );
}

/// 记录可疑登录
pub fn log_suspicious_login(user_id: &str, tenant_id: &str, reason: &str, ip: &str) {
    warn!(
        user_id = user_id,
        tenant_id = tenant_id,
        reason = reason,
        ip = ip,
        "Suspicious login detected"
    );
}

/// 记录 2FA 验证
pub fn log_2fa_verification(user_id: &str, tenant_id: &str, method: &str, success: bool) {
    if success {
        info!(
            user_id = user_id,
            tenant_id = tenant_id,
            method = method,
            "2FA verification successful"
        );
    } else {
        warn!(
            user_id = user_id,
            tenant_id = tenant_id,
            method = method,
            "2FA verification failed"
        );
    }
}

/// 记录密码重置
pub fn log_password_reset(user_id: &str, tenant_id: &str) {
    info!(
        user_id = user_id,
        tenant_id = tenant_id,
        "Password reset completed"
    );
}

/// 记录 OAuth2 授权
pub fn log_oauth_authorization(client_id: &str, user_id: &str, scope: &str, success: bool) {
    if success {
        info!(
            client_id = client_id,
            user_id = user_id,
            scope = scope,
            "OAuth2 authorization granted"
        );
    } else {
        warn!(
            client_id = client_id,
            user_id = user_id,
            scope = scope,
            "OAuth2 authorization denied"
        );
    }
}

/// 记录数据库错误
pub fn log_db_error(operation: &str, table: &str, error: &str) {
    error!(
        operation = operation,
        table = table,
        error = error,
        "Database operation failed"
    );
}

/// 记录缓存错误
pub fn log_cache_error(operation: &str, error: &str) {
    error!(
        operation = operation,
        error = error,
        "Cache operation failed"
    );
}

// ============================================================================
// 性能分析辅助函数
// ============================================================================

/// 记录慢查询
pub fn log_slow_query(operation: &str, table: &str, duration_ms: f64) {
    warn!(
        operation = operation,
        table = table,
        duration_ms = duration_ms,
        "Slow database query detected"
    );
}

/// 记录慢 API 请求
pub fn log_slow_api_request(method: &str, duration_ms: f64) {
    warn!(
        method = method,
        duration_ms = duration_ms,
        "Slow API request detected"
    );
}

// ============================================================================
// 调试日志
// ============================================================================

/// 记录调试信息
pub fn log_debug(message: &str, context: &[(&str, &str)]) {
    debug!(?context, "{}", message);
}
