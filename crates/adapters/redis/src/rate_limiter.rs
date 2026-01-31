//! 限流器模块
//!
//! 提供基于 Redis 的分布式限流功能

use errors::{AppError, AppResult};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Script};
use tracing::debug;

/// 限流结果
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// 是否允许
    pub allowed: bool,
    /// 剩余配额
    pub remaining: i64,
    /// 重置时间（秒）
    pub reset_after_secs: i64,
    /// 总配额
    pub limit: i64,
}

impl RateLimitResult {
    /// 是否被限流
    pub fn is_limited(&self) -> bool {
        !self.allowed
    }
}

/// 限流器类型
#[derive(Debug, Clone, Copy)]
pub enum RateLimiterType {
    /// 固定窗口
    FixedWindow,
    /// 滑动窗口
    SlidingWindow,
    /// 令牌桶
    TokenBucket,
}

/// Redis 限流器
pub struct RedisRateLimiter {
    conn: ConnectionManager,
    key_prefix: String,
    limiter_type: RateLimiterType,
}

impl RedisRateLimiter {
    /// 创建新的限流器
    pub fn new(conn: ConnectionManager) -> Self {
        Self {
            conn,
            key_prefix: "ratelimit".to_string(),
            limiter_type: RateLimiterType::SlidingWindow,
        }
    }

    /// 设置键前缀
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    /// 设置限流器类型
    pub fn with_type(mut self, limiter_type: RateLimiterType) -> Self {
        self.limiter_type = limiter_type;
        self
    }

    /// 生成限流键
    fn rate_limit_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    /// 检查并消费配额（固定窗口）
    pub async fn check_fixed_window(
        &mut self,
        key: &str,
        limit: i64,
        window_secs: i64,
    ) -> AppResult<RateLimitResult> {
        let rate_key = self.rate_limit_key(key);

        // 使用 Lua 脚本确保原子性
        let script = Script::new(
            r#"
            local key = KEYS[1]
            local limit = tonumber(ARGV[1])
            local window = tonumber(ARGV[2])

            local current = redis.call('INCR', key)

            if current == 1 then
                redis.call('EXPIRE', key, window)
            end

            local ttl = redis.call('TTL', key)
            if ttl < 0 then
                ttl = window
            end

            local allowed = current <= limit
            local remaining = limit - current
            if remaining < 0 then
                remaining = 0
            end

            return {allowed and 1 or 0, remaining, ttl, limit}
            "#,
        );

        let result: Vec<i64> = script
            .key(&rate_key)
            .arg(limit)
            .arg(window_secs)
            .invoke_async(&mut self.conn)
            .await
            .map_err(|e| AppError::internal(format!("Rate limit check failed: {}", e)))?;

        let rate_result = RateLimitResult {
            allowed: result[0] == 1,
            remaining: result[1],
            reset_after_secs: result[2],
            limit: result[3],
        };

        debug!(
            key = %key,
            allowed = rate_result.allowed,
            remaining = rate_result.remaining,
            "Fixed window rate limit check"
        );

        Ok(rate_result)
    }

    /// 检查并消费配额（滑动窗口）
    pub async fn check_sliding_window(
        &mut self,
        key: &str,
        limit: i64,
        window_secs: i64,
    ) -> AppResult<RateLimitResult> {
        let rate_key = self.rate_limit_key(key);
        let now = chrono::Utc::now().timestamp_millis();
        let window_ms = window_secs * 1000;

        // 使用 Lua 脚本实现滑动窗口
        let script = Script::new(
            r#"
            local key = KEYS[1]
            local limit = tonumber(ARGV[1])
            local window_ms = tonumber(ARGV[2])
            local now = tonumber(ARGV[3])
            local window_start = now - window_ms

            -- 移除过期的请求
            redis.call('ZREMRANGEBYSCORE', key, '-inf', window_start)

            -- 获取当前窗口内的请求数
            local current = redis.call('ZCARD', key)

            local allowed = current < limit

            if allowed then
                -- 添加当前请求
                redis.call('ZADD', key, now, now .. '-' .. math.random(1000000))
                current = current + 1
            end

            -- 设置过期时间
            redis.call('PEXPIRE', key, window_ms)

            local remaining = limit - current
            if remaining < 0 then
                remaining = 0
            end

            -- 计算重置时间
            local oldest = redis.call('ZRANGE', key, 0, 0, 'WITHSCORES')
            local reset_after = window_ms
            if #oldest >= 2 then
                reset_after = tonumber(oldest[2]) + window_ms - now
                if reset_after < 0 then
                    reset_after = 0
                end
            end

            return {allowed and 1 or 0, remaining, math.ceil(reset_after / 1000), limit}
            "#,
        );

        let result: Vec<i64> = script
            .key(&rate_key)
            .arg(limit)
            .arg(window_ms)
            .arg(now)
            .invoke_async(&mut self.conn)
            .await
            .map_err(|e| AppError::internal(format!("Rate limit check failed: {}", e)))?;

        let rate_result = RateLimitResult {
            allowed: result[0] == 1,
            remaining: result[1],
            reset_after_secs: result[2],
            limit: result[3],
        };

        debug!(
            key = %key,
            allowed = rate_result.allowed,
            remaining = rate_result.remaining,
            "Sliding window rate limit check"
        );

        Ok(rate_result)
    }

    /// 检查并消费配额（令牌桶）
    pub async fn check_token_bucket(
        &mut self,
        key: &str,
        capacity: i64,
        refill_rate: i64, // 每秒补充的令牌数
        tokens_requested: i64,
    ) -> AppResult<RateLimitResult> {
        let rate_key = self.rate_limit_key(key);
        let now = chrono::Utc::now().timestamp_millis();

        // 使用 Lua 脚本实现令牌桶
        let script = Script::new(
            r#"
            local key = KEYS[1]
            local capacity = tonumber(ARGV[1])
            local refill_rate = tonumber(ARGV[2])
            local tokens_requested = tonumber(ARGV[3])
            local now = tonumber(ARGV[4])

            local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
            local tokens = tonumber(bucket[1]) or capacity
            local last_refill = tonumber(bucket[2]) or now

            -- 计算补充的令牌
            local elapsed_ms = now - last_refill
            local refill = math.floor(elapsed_ms * refill_rate / 1000)
            tokens = math.min(capacity, tokens + refill)

            local allowed = tokens >= tokens_requested

            if allowed then
                tokens = tokens - tokens_requested
            end

            -- 更新桶状态
            redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
            redis.call('EXPIRE', key, math.ceil(capacity / refill_rate) + 1)

            -- 计算重置时间（补满所需时间）
            local tokens_needed = capacity - tokens
            local reset_after = 0
            if tokens_needed > 0 and refill_rate > 0 then
                reset_after = math.ceil(tokens_needed / refill_rate)
            end

            return {allowed and 1 or 0, tokens, reset_after, capacity}
            "#,
        );

        let result: Vec<i64> = script
            .key(&rate_key)
            .arg(capacity)
            .arg(refill_rate)
            .arg(tokens_requested)
            .arg(now)
            .invoke_async(&mut self.conn)
            .await
            .map_err(|e| AppError::internal(format!("Rate limit check failed: {}", e)))?;

        let rate_result = RateLimitResult {
            allowed: result[0] == 1,
            remaining: result[1],
            reset_after_secs: result[2],
            limit: result[3],
        };

        debug!(
            key = %key,
            allowed = rate_result.allowed,
            remaining = rate_result.remaining,
            "Token bucket rate limit check"
        );

        Ok(rate_result)
    }

    /// 通用检查方法（根据配置的类型）
    pub async fn check(
        &mut self,
        key: &str,
        limit: i64,
        window_secs: i64,
    ) -> AppResult<RateLimitResult> {
        match self.limiter_type {
            RateLimiterType::FixedWindow => {
                self.check_fixed_window(key, limit, window_secs).await
            }
            RateLimiterType::SlidingWindow => {
                self.check_sliding_window(key, limit, window_secs).await
            }
            RateLimiterType::TokenBucket => {
                // 对于令牌桶，将 limit 作为容量，window_secs 作为补充周期
                let refill_rate = limit / window_secs.max(1);
                self.check_token_bucket(key, limit, refill_rate.max(1), 1).await
            }
        }
    }

    /// 重置限流计数
    pub async fn reset(&mut self, key: &str) -> AppResult<()> {
        let rate_key = self.rate_limit_key(key);
        self.conn
            .del::<_, ()>(&rate_key)
            .await
            .map_err(|e| AppError::internal(format!("Failed to reset rate limit: {}", e)))?;
        Ok(())
    }

    /// 获取当前计数
    pub async fn get_count(&mut self, key: &str) -> AppResult<i64> {
        let rate_key = self.rate_limit_key(key);

        match self.limiter_type {
            RateLimiterType::FixedWindow => {
                let count: Option<i64> = self
                    .conn
                    .get(&rate_key)
                    .await
                    .map_err(|e| AppError::internal(format!("Failed to get count: {}", e)))?;
                Ok(count.unwrap_or(0))
            }
            RateLimiterType::SlidingWindow => {
                let count: i64 = self
                    .conn
                    .zcard(&rate_key)
                    .await
                    .map_err(|e| AppError::internal(format!("Failed to get count: {}", e)))?;
                Ok(count)
            }
            RateLimiterType::TokenBucket => {
                let tokens: Option<i64> = self
                    .conn
                    .hget(&rate_key, "tokens")
                    .await
                    .map_err(|e| AppError::internal(format!("Failed to get tokens: {}", e)))?;
                Ok(tokens.unwrap_or(0))
            }
        }
    }
}

/// 简单的限流检查（固定窗口）
pub async fn check_rate_limit(
    conn: &mut ConnectionManager,
    key: &str,
    limit: i64,
    window_secs: i64,
) -> AppResult<RateLimitResult> {
    let mut limiter = RedisRateLimiter::new(conn.clone())
        .with_type(RateLimiterType::FixedWindow);
    limiter.check_fixed_window(key, limit, window_secs).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_result() {
        let result = RateLimitResult {
            allowed: true,
            remaining: 9,
            reset_after_secs: 60,
            limit: 10,
        };

        assert!(!result.is_limited());
        assert_eq!(result.remaining, 9);

        let limited = RateLimitResult {
            allowed: false,
            remaining: 0,
            reset_after_secs: 30,
            limit: 10,
        };

        assert!(limited.is_limited());
    }

    #[tokio::test]
    #[ignore] // 需要 Redis 实例
    async fn test_fixed_window_rate_limiter() {
        let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let conn = redis::aio::ConnectionManager::new(client).await.unwrap();

        let mut limiter = RedisRateLimiter::new(conn)
            .with_key_prefix("test")
            .with_type(RateLimiterType::FixedWindow);

        // 重置
        limiter.reset("test-key").await.unwrap();

        // 第一次请求应该允许
        let result = limiter.check_fixed_window("test-key", 5, 60).await.unwrap();
        assert!(result.allowed);
        assert_eq!(result.remaining, 4);

        // 继续请求直到限流
        for _ in 0..4 {
            limiter.check_fixed_window("test-key", 5, 60).await.unwrap();
        }

        // 第 6 次请求应该被限流
        let result = limiter.check_fixed_window("test-key", 5, 60).await.unwrap();
        assert!(!result.allowed);
        assert_eq!(result.remaining, 0);

        // 清理
        limiter.reset("test-key").await.unwrap();
    }
}
