//! 通用重试机制模块
//!
//! 提供带指数退避的重试逻辑，可被各适配器复用

use std::future::Future;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// 通用重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_attempts: u32,
    /// 初始延迟
    pub initial_delay: Duration,
    /// 最大延迟
    pub max_delay: Duration,
    /// 退避乘数
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
}

fn default_multiplier() -> f64 {
    2.0
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// 创建新的重试配置
    pub fn new(max_attempts: u32, initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay,
            multiplier: 2.0,
        }
    }

    /// 设置退避乘数
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// 计算第 n 次重试的延迟
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms =
            self.initial_delay.as_millis() as f64 * self.multiplier.powi(attempt as i32);
        let capped_delay = (delay_ms as u64).min(self.max_delay.as_millis() as u64);
        Duration::from_millis(capped_delay)
    }
}

/// 通用可重试错误模式
pub const COMMON_RETRYABLE_PATTERNS: &[&str] = &[
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
];

/// 判断错误是否可重试（通用版本）
pub fn is_retryable_error(error: &str) -> bool {
    let error_lower = error.to_lowercase();
    COMMON_RETRYABLE_PATTERNS
        .iter()
        .any(|pattern| error_lower.contains(pattern))
}

/// 带重试的异步操作执行器
///
/// # 参数
/// - `config`: 重试配置
/// - `operation_name`: 操作名称（用于日志）
/// - `operation`: 要执行的异步操作
///
/// # 返回
/// 操作成功时返回 Ok(T)，所有重试都失败时返回最后一次的错误
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
                        "Operation succeeded after retry"
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
                        "Operation failed, no more retries"
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
                        "Operation failed, retrying"
                    );
                    tokio::time::sleep(delay).await;
                    last_error = Some(e);
                }
            }
        }
    }

    // 使用 unwrap_or_else 替代 expect，更安全的错误处理
    Err(last_error.unwrap_or_else(|| unreachable!("loop guarantees at least one attempt")))
}

/// 带条件重试的异步操作执行器
///
/// 允许自定义重试条件，只有当 `should_retry` 返回 true 时才会重试
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
                        "Operation succeeded after retry"
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
                            "Operation failed with non-retryable error"
                        );
                    } else {
                        warn!(
                            operation = operation_name,
                            attempt = attempt + 1,
                            max_attempts = config.max_attempts,
                            error = %e,
                            "Operation failed, no more retries"
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
                    "Operation failed, retrying"
                );
                tokio::time::sleep(delay).await;
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| unreachable!("loop guarantees at least one attempt")))
}

/// 带重试的异步操作执行器（可选组件版本）
///
/// 与 `with_retry` 类似，但失败时返回 None 而不是错误
pub async fn with_retry_optional<F, Fut, T, E>(
    config: &RetryConfig,
    operation_name: &str,
    operation: F,
) -> Option<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    match with_retry(config, operation_name, operation).await {
        Ok(result) => Some(result),
        Err(e) => {
            warn!(
                operation = operation_name,
                error = %e,
                "Optional operation failed after all retries, continuing without it"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let config = RetryConfig::new(3, Duration::from_millis(10), Duration::from_millis(100));
        let result: Result<i32, &str> = with_retry(&config, "test", || async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let config = RetryConfig::new(3, Duration::from_millis(10), Duration::from_millis(100));
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
        let config = RetryConfig::new(3, Duration::from_millis(10), Duration::from_millis(100));
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
    async fn test_conditional_retry() {
        let config = RetryConfig::new(5, Duration::from_millis(10), Duration::from_millis(100));
        let counter = AtomicU32::new(0);

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
        let config = RetryConfig::new(5, Duration::from_millis(10), Duration::from_millis(100));
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

    #[tokio::test]
    async fn test_retry_optional_success() {
        let config = RetryConfig::new(3, Duration::from_millis(10), Duration::from_millis(100));
        let result: Option<i32> =
            with_retry_optional(&config, "test", || async { Ok::<_, &str>(42) }).await;
        assert_eq!(result, Some(42));
    }

    #[tokio::test]
    async fn test_retry_optional_failure() {
        let config = RetryConfig::new(3, Duration::from_millis(10), Duration::from_millis(100));
        let result: Option<i32> =
            with_retry_optional(&config, "test", || async { Err::<i32, _>("error") }).await;
        assert_eq!(result, None);
    }

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig::new(5, Duration::from_millis(100), Duration::from_secs(5));

        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(800));
        // Should be capped at max_delay
        assert_eq!(config.delay_for_attempt(10), Duration::from_secs(5));
    }

    #[test]
    fn test_is_retryable_error() {
        assert!(is_retryable_error("connection refused"));
        assert!(is_retryable_error("Connection timed out"));
        assert!(is_retryable_error("econnrefused"));
        assert!(is_retryable_error("server is busy"));
        assert!(is_retryable_error("broken pipe"));
        assert!(!is_retryable_error("key not found"));
        assert!(!is_retryable_error("wrong type"));
        assert!(!is_retryable_error("syntax error"));
    }
}
