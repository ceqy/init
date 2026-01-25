//! 重试工具模块
//!
//! 提供指数退避重试逻辑

use std::future::Future;
use std::time::Duration;

use tracing::{info, warn};

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_attempts: u32,
    /// 初始延迟（毫秒）
    pub initial_delay_ms: u64,
    /// 最大延迟（毫秒）
    pub max_delay_ms: u64,
    /// 退避乘数
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// 创建新的重试配置
    pub fn new(max_attempts: u32, initial_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_attempts,
            initial_delay_ms,
            max_delay_ms,
            multiplier: 2.0,
        }
    }

    /// 设置退避乘数
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// 计算第 n 次重试的延迟
    fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms = (self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32)) as u64;
        let capped_delay = delay_ms.min(self.max_delay_ms);
        Duration::from_millis(capped_delay)
    }
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

    Err(last_error.expect("at least one attempt should have been made"))
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
        let config = RetryConfig::new(3, 100, 1000);
        let result: Result<i32, &str> =
            with_retry(&config, "test", || async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let config = RetryConfig::new(3, 10, 100);
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
        let config = RetryConfig::new(3, 10, 100);
        let counter = AtomicU32::new(0);

        let result: Result<i32, &str> = with_retry(&config, "test", || {
            counter.fetch_add(1, Ordering::SeqCst);
            async { Err("permanent error") }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig::new(5, 1000, 30000);

        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(1000));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(2000));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(4000));
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(8000));
        assert_eq!(config.delay_for_attempt(4), Duration::from_millis(16000));
        assert_eq!(config.delay_for_attempt(5), Duration::from_millis(30000)); // capped
    }
}
