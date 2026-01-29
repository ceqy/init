//! 自定义断路器实现
//!
//! 提供基本的断路器功能：Closed -> Open -> HalfOpen 状态转换

use cuba_errors::{AppError, AppResult};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
struct CircuitBreakerState {
    state: State,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitBreakerState>>,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub reset_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            reset_timeout: Duration::from_secs(30),
        }
    }
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitBreakerState {
                state: State::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
            })),
        }
    }

    /// 执行受保护的操作
    pub async fn call<F, Fut, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = AppResult<T>>,
    {
        // 1. 检查状态
        self.check_state()?;

        // 2. 执行操作
        let result = f().await;

        // 3. 记录结果
        self.record_result(result.is_ok());

        result
    }

    fn check_state(&self) -> AppResult<()> {
        let mut state = self.state.lock().unwrap();

        match state.state {
            State::Closed => Ok(()),
            State::Open => {
                if let Some(last_failure) = state.last_failure_time {
                    if last_failure.elapsed() >= self.config.reset_timeout {
                        // 尝试转为 HalfOpen
                        state.state = State::HalfOpen;
                        state.success_count = 0;
                        return Ok(());
                    }
                }
                Err(AppError::external_service("Circuit breaker is open"))
            }
            State::HalfOpen => {
                // HalfOpen 允许一次尝试 (简单的实现：允许所有，直到失败或成功阈值)
                // 更严格的实现应该限制并发
                Ok(())
            }
        }
    }

    fn record_result(&self, success: bool) {
        let mut state = self.state.lock().unwrap();

        if success {
            match state.state {
                State::Closed => {
                    state.failure_count = 0;
                }
                State::Open => {
                    // 理论上不会发生，因为 Open 不会执行
                }
                State::HalfOpen => {
                    state.success_count += 1;
                    if state.success_count >= self.config.success_threshold {
                        state.state = State::Closed;
                        state.failure_count = 0;
                        state.success_count = 0;
                        tracing::info!("Circuit breaker transitioned to CLOSED");
                    }
                }
            }
        } else {
            match state.state {
                State::Closed => {
                    state.failure_count += 1;
                    if state.failure_count >= self.config.failure_threshold {
                        state.state = State::Open;
                        state.last_failure_time = Some(Instant::now());
                        tracing::warn!("Circuit breaker transitioned to OPEN");
                    }
                }
                State::Open => {
                    // 更新时间？通常不需要
                }
                State::HalfOpen => {
                    state.state = State::Open;
                    state.last_failure_time = Some(Instant::now());
                    tracing::warn!("Circuit breaker transitioned back to OPEN from HalfOpen");
                }
            }
        }
    }
}
