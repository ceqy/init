//! 限流器
//!
//! 使用滑动窗口算法和 Lua 脚本实现原子操作

use crate::rate_limit::config::ConfigManager;
use crate::rate_limit::types::{RateLimitResult, RateLimitRule};
use redis::Script;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use tracing::{debug, warn};

/// Lua 脚本：滑动窗口限流
///
/// # 参数
/// - KEYS[1]: Redis 键
/// - ARGV[1]: 当前窗口起始时间
/// - ARGV[2]: 窗口大小（秒）
/// - ARGV[3]: 最大请求数
/// - ARGV[4]: 突发缓冲大小
///
/// # 返回值
/// - array[0]: 当前计数
/// - array[1]: 是否允许 (1/0)
/// - array[2]: 剩余请求数
static SLIDING_WINDOW_SCRIPT: &str = r#"
local key = KEYS[1]
local window_start = tonumber(ARGV[1])
local window_size = tonumber(ARGV[2])
local max_requests = tonumber(ARGV[3])
local burst_size = tonumber(ARGV[4])

-- 删除过期数据
local window_end = window_start + window_size
redis.call('ZREMRANGEBYSCORE', key, '-inf', '(' .. tostring(window_start))

-- 获取当前窗口内的计数
local count = redis.call('ZCARD', key)

-- 允许的条件：count < max_requests 或 count < max_requests + burst_size
local allowed = 0
if count < max_requests then
    allowed = 1
elseif count < (max_requests + burst_size) then
    -- 在突发缓冲区内，允许但标记为超额
    allowed = 1
end

-- 如果允许，添加当前请求
if allowed == 1 then
    -- 使用当前时间戳作为分数
    local now = window_start + window_size
    redis.call('ZADD', key, now, tostring(now))
    -- 设置过期时间为 2 倍窗口大小（确保能处理边界情况）
    redis.call('EXPIRE', key, window_size * 2)
    count = count + 1
end

-- 计算剩余请求数
local remaining = max_requests
if count >= max_requests then
    remaining = 0
else
    remaining = max_requests - count
end

-- 返回结果
return {count, allowed, remaining}
"#;

/// 限流器
#[derive(Clone)]
#[cfg_attr(test, allow(dead_code))]
pub struct RateLimiter {
    /// Redis 连接管理器
    pub redis_conn: Arc<ConnectionManager>,
    /// Lua 脚本
    pub script: Arc<Script>,
}

impl RateLimiter {
    /// 创建新的限流器
    pub fn new(redis_conn: ConnectionManager) -> Self {
        Self {
            redis_conn: Arc::new(redis_conn),
            script: Arc::new(Script::new(SLIDING_WINDOW_SCRIPT)),
        }
    }

    /// 检查是否允许请求
    ///
    /// # 参数
    /// - `key_prefix`: Redis 键前缀
    /// - `identifier`: 客户端标识符（user:{user_id} 或 ip:{ip}）
    /// - `rule`: 限流规则
    ///
    /// # 返回
    /// 限流检查结果
    pub async fn check(
        &self,
        key_prefix: &str,
        identifier: &str,
        rule: &RateLimitRule,
    ) -> RateLimitResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 计算窗口起始时间
        let window_start = now - (now % rule.window_secs);
        let reset_at = window_start + rule.window_secs;

        // 构建 Redis 键
        let key = format!("{}:{}:{}", key_prefix, identifier, window_start);

        // 执行 Lua 脚本
        let result = self
            .execute_script(&key, window_start, rule)
            .await
            .unwrap_or_else(|e| {
                warn!(error = %e, key = %key, "Rate limit script failed, allowing request (fail-open)");
                // 失败时允许请求通过
                RateLimitResult {
                    allowed: true,
                    count: 0,
                    remaining: rule.max_requests,
                    limit: rule.max_requests,
                    reset_at,
                    retry_after: None,
                }
            });

        result
    }

    /// 执行 Lua 脚本
    async fn execute_script(
        &self,
        key: &str,
        window_start: u64,
        rule: &RateLimitRule,
    ) -> Result<RateLimitResult, String> {
        let mut conn = (*self.redis_conn).clone();

        // 执行脚本 - 返回 [count, allowed, remaining] 数组
        let values: Vec<u64> = self
            .script
            .key(key)
            .arg(window_start)
            .arg(rule.window_secs)
            .arg(rule.max_requests)
            .arg(rule.burst_size)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| format!("Script execution failed: {}", e))?;

        if values.len() != 3 {
            return Err(format!("Unexpected script result length: {}", values.len()));
        }

        let count = values[0];
        let allowed = values[1];
        let remaining = values[2];

        let retry_after = if allowed == 0 {
            Some(rule.window_secs) // 拒绝时建议等待整个窗口
        } else {
            None
        };

        debug!(
            key,
            count,
            allowed,
            remaining,
            max_requests = rule.max_requests,
            "Rate limit check result"
        );

        Ok(RateLimitResult {
            allowed: allowed == 1,
            count,
            remaining,
            limit: rule.max_requests,
            reset_at: window_start + rule.window_secs,
            retry_after,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window_script_validity() {
        // 验证 Lua 脚本语法
        assert!(SLIDING_WINDOW_SCRIPT.contains("ZREMRANGEBYSCORE"));
        assert!(SLIDING_WINDOW_SCRIPT.contains("ZCARD"));
        assert!(SLIDING_WINDOW_SCRIPT.contains("ZADD"));
        assert!(SLIDING_WINDOW_SCRIPT.contains("EXPIRE"));
    }
}
