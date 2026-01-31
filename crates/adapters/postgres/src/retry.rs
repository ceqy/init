//! 重试机制模块
//!
//! 提供带指数退避的重试逻辑

use std::future::Future;

use cuba_errors::AppResult;
use tracing::{info, warn};

use crate::config::RetryConfig;

/// 带重试的异步操作执行器
pub async fn with_retry<F, Fut, T, E>(
    config: &RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error: Option<E> = None;

    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    info!(
                        operation = operation_name,
                        attempt = attempt + 1,
                        "PostgreSQL operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                let is_last_attempt = attempt + 1 >= config.max_attempts;

                if is_last_attempt {
                    warn!(
                        operation = operation_name,
                        attempt = attempt + 1,
                        max_attempts = config.max_attempts,
                        error = %e,
                        "PostgreSQL operation failed, no more retries"
                    );
                    last_error = Some(e);
                } else {
                    let delay = config.delay_for_attempt(attempt);
                    warn!(
                        operation = operation_name,
                        attempt = attempt + 1,
                        max_attempts = config.max_attempts,
                        error = %e,
                        delay_ms = delay.as_millis(),
                        "PostgreSQL operation failed, retrying"
                    );
                    tokio::time::sleep(delay).await;
                    last_error = Some(e);
                }
            }
        }
    }

    Err(last_error.expect("at least one attempt should have been made"))
}

/// 带重试的异步操作执行器（返回 AppResult）
pub async fn with_retry_app<F, Fut, T>(
    config: &RetryConfig,
    operation_name: &str,
    operation: F,
) -> AppResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = AppResult<T>>,
{
    with_retry(config, operation_name, operation).await
}

/// 判断错误是否可重试
pub fn is_retryable_error(error: &str) -> bool {
    let retryable_patterns = [
        "connection refused",
        "connection reset",
        "connection timed out",
        "timeout",
        "temporarily unavailable",
        "too many connections",
        "server is busy",
        "network",
        "econnrefused",
        "etimedout",
        "econnreset",
        "broken pipe",
        "connection closed",
        "eof",
        "ssl",
        "could not connect",
        "no route to host",
        "connection terminated",
        "server closed the connection",
        "canceling statement due to statement timeout",
        "deadlock detected",
        "serialization failure",
    ];

    let error_lower = error.to_lowercase();
    retryable_patterns
        .iter()
        .any(|pattern| error_lower.contains(pattern))
}

/// 判断 sqlx 错误是否可重试
pub fn is_sqlx_retryable(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Io(_) => true,
        sqlx::Error::Tls(_) => true,
        sqlx::Error::PoolTimedOut => true,
        sqlx::Error::PoolClosed => false,
        sqlx::Error::WorkerCrashed => true,
        sqlx::Error::Database(db_err) => {
            // PostgreSQL 错误码
            // 40001: serialization_failure
            // 40P01: deadlock_detected
            // 57P01: admin_shutdown
            // 57P02: crash_shutdown
            // 57P03: cannot_connect_now
            // 08000: connection_exception
            // 08003: connection_does_not_exist
            // 08006: connection_failure
            if let Some(code) = db_err.code() {
                let code_str = code.as_ref();
                matches!(
                    code_str,
                    "40001" | "40P01" | "57P01" | "57P02" | "57P03" | "08000" | "08003" | "08006"
                )
            } else {
                is_retryable_error(&db_err.to_string())
            }
        }
        _ => is_retryable_error(&error.to_string()),
    }
}

/// 带条件重试的异步操作执行器
pub async fn with_conditional_retry<F, Fut, T, E, P>(
    config: &RetryConfig,
    operation_name: &str,
    mut operation: F,
    should_retry: P,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    P: Fn(&E) -> bool,
{
    let mut last_error: Option<E> = None;

    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    info!(
                        operation = operation_name,
                        attempt = attempt + 1,
                        "PostgreSQL operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                let is_last_attempt = attempt + 1 >= config.max_attempts;
                let can_retry = should_retry(&e);

                if is_last_attempt || !can_retry {
                    if !can_retry {
                        warn!(
                            operation = operation_name,
                            attempt = attempt + 1,
                            error = %e,
                            "PostgreSQL operation failed with non-retryable error"
                        );
                    } else {
                        warn!(
                            operation = operation_name,
                            attempt = attempt + 1,
                            max_attempts = config.max_attempts,
                            error = %e,
                            "PostgreSQL operation failed, no more retries"
                        );
                    }
                    return Err(e);
                }

                let delay = config.delay_for_attempt(attempt);
                warn!(
                    operation = operation_name,
                    attempt = attempt + 1,
                    max_attempts = config.max_attempts,
                    error = %e,
                    delay_ms = delay.as_millis(),
                    "PostgreSQL operation failed, retrying"
                );
                tokio::time::sleep(delay).await;
                last_error = Some(e);
            }
        }
    }

    Err(last_error.expect("at least one attempt should have been made"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuba_errors::AppError;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let config = RetryConfig::new(
            3,
            Duration::from_millis(10),
            Duration::from_millis(100),
        );
        let result: Result<i32, &str> =
            with_retry(&config, "test", || async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let config = RetryConfig::new(
            3,
            Duration::from_millis(10),
            Duration::from_millis(100),
        );
        let counter = AtomicU32::new(0);

        let result: Result<i32, &str> = with_retry(&config, "test", || {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    Err("temporary error")
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_all_failures() {
        let config = RetryConfig::new(
            3,
            Duration::from_millis(10),
            Duration::from_millis(100),
        );
        let counter = AtomicU32::new(0);

        let result: Result<i32, &str> = with_retry(&config, "test", || {
            counter.fetch_add(1, Ordering::SeqCst);
            async { Err("permanent error") }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_app_result() {
        let config = RetryConfig::new(
            3,
            Duration::from_millis(10),
            Duration::from_millis(100),
        );

        let result: AppResult<i32> =
            with_retry_app(&config, "test", || async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);

        let result: AppResult<i32> = with_retry_app(&config, "test", || async {
            Err(AppError::database("test error"))
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_conditional_retry() {
        let config = RetryConfig::new(
            5,
            Duration::from_millis(10),
            Duration::from_millis(100),
        );
        let counter = AtomicU32::new(0);

        // 只重试 "retryable" 错误
        let result: Result<i32, &str> = with_conditional_retry(
            &config,
            "test",
            || {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                async move {
                    if count < 2 {
                        Err("retryable error")
                    } else {
                        Ok(42)
                    }
                }
            },
            |e| e.contains("retryable"),
        )
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_conditional_retry_non_retryable() {
        let config = RetryConfig::new(
            5,
            Duration::from_millis(10),
            Duration::from_millis(100),
        );
        let counter = AtomicU32::new(0);

        let result: Result<i32, &str> = with_conditional_retry(
            &config,
            "test",
            || {
                counter.fetch_add(1, Ordering::SeqCst);
                async { Err("permanent error") }
            },
            |e| e.contains("retryable"),
        )
        .await;

        assert!(result.is_err());
        // 应该只尝试一次，因为错误不可重试
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_is_retryable_error() {
        assert!(is_retryable_error("connection refused"));
        assert!(is_retryable_error("Connection timed out"));
        assert!(is_retryable_error("econnrefused"));
        assert!(is_retryable_error("server is busy"));
        assert!(is_retryable_error("broken pipe"));
        assert!(is_retryable_error("deadlock detected"));
        assert!(is_retryable_error("serialization failure"));
        assert!(!is_retryable_error("unique constraint violation"));
        assert!(!is_retryable_error("foreign key violation"));
    }
}
