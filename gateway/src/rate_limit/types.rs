//! 数据结构定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 用户等级（基于 JWT permissions）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserTier {
    /// 匿名用户（未认证）
    Anonymous,
    /// 标准用户（已认证，无特殊权限）
    Standard,
    /// 高级用户（有 `rate_limit:premium` 权限）
    Premium,
    /// VIP 用户（有 `rate_limit:vip` 权限）
    Vip,
}

impl UserTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Anonymous => "anonymous",
            Self::Standard => "standard",
            Self::Premium => "premium",
            Self::Vip => "vip",
        }
    }
}

/// 接口类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EndpointType {
    /// 认证接口（login, register, refresh 等）
    Auth,
    /// 查询接口（GET 请求）
    Query,
    /// 写入接口（POST, PUT, DELETE）
    Write,
    /// 管理接口（/api/admin/*）
    Admin,
}

impl EndpointType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auth => "auth",
            Self::Query => "query",
            Self::Write => "write",
            Self::Admin => "admin",
        }
    }
}

/// 单个限流规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    /// 时间窗口内的最大请求数
    pub max_requests: u64,
    /// 时间窗口（秒）
    pub window_secs: u64,
    /// 突发流量缓冲大小（允许短暂超限）
    pub burst_size: u64,
}

impl Default for RateLimitRule {
    fn default() -> Self {
        Self {
            max_requests: 60,
            window_secs: 60,
            burst_size: 10,
        }
    }
}

/// 限流配置（从 Redis 加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 是否启用限流
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Redis 键前缀
    #[serde(default = "default_redis_key_prefix")]
    pub redis_key_prefix: String,
    /// 默认规则（当没有匹配到特定规则时使用）
    #[serde(default)]
    pub default_rule: RateLimitRule,
    /// 分级规则映射（key: "tier:endpoint_type"）
    #[serde(default)]
    pub rules: HashMap<String, RateLimitRule>,
}

fn default_enabled() -> bool {
    true
}

fn default_redis_key_prefix() -> String {
    "rl".to_string()
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            redis_key_prefix: "rl".to_string(),
            default_rule: RateLimitRule::default(),
            rules: HashMap::new(),
        }
    }
}

/// 限流检查结果
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// 是否允许请求
    pub allowed: bool,
    /// 当前窗口内的请求计数
    pub count: u64,
    /// 剩余可用请求数
    pub remaining: u64,
    /// 限制的最大请求数
    pub limit: u64,
    /// 窗口重置时间（Unix 时间戳）
    pub reset_at: u64,
    /// 建议重试等待时间（秒，仅在拒绝时有效）
    pub retry_after: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_tier_as_str() {
        assert_eq!(UserTier::Anonymous.as_str(), "anonymous");
        assert_eq!(UserTier::Standard.as_str(), "standard");
        assert_eq!(UserTier::Premium.as_str(), "premium");
        assert_eq!(UserTier::Vip.as_str(), "vip");
    }

    #[test]
    fn test_endpoint_type_as_str() {
        assert_eq!(EndpointType::Auth.as_str(), "auth");
        assert_eq!(EndpointType::Query.as_str(), "query");
        assert_eq!(EndpointType::Write.as_str(), "write");
        assert_eq!(EndpointType::Admin.as_str(), "admin");
    }

    #[test]
    fn test_rate_limit_rule_default() {
        let rule = RateLimitRule::default();
        assert_eq!(rule.max_requests, 60);
        assert_eq!(rule.window_secs, 60);
        assert_eq!(rule.burst_size, 10);
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert!(config.enabled);
        assert_eq!(config.redis_key_prefix, "rl");
    }

    #[test]
    fn test_rate_limit_config_serde() {
        let config = RateLimitConfig {
            enabled: true,
            redis_key_prefix: "rl".to_string(),
            default_rule: RateLimitRule::default(),
            rules: {
                let mut map = HashMap::new();
                map.insert(
                    "anonymous:auth".to_string(),
                    RateLimitRule {
                        max_requests: 10,
                        window_secs: 60,
                        burst_size: 2,
                    },
                );
                map
            },
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RateLimitConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.rules["anonymous:auth"].max_requests, 10);
    }
}
