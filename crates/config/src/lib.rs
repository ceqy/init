//! cuba-config - 配置加载库

use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;
use thiserror::Error;

use secrecy::Secret;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load config: {0}")]
    Load(#[from] Box<figment::Error>),
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: Secret<String>,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// 可选: 读库 URL（用于读写分离）
    pub read_url: Option<Secret<String>>,
    /// 可选: 读库最大连接数
    #[serde(default = "default_read_max_connections")]
    pub read_max_connections: u32,
}

fn default_max_connections() -> u32 {
    // 根据环境自动调整连接池大小
    // 开发环境: 10, 生产环境: 50
    match std::env::var("APP_ENV").as_deref() {
        Ok("production") => 50,
        _ => 10,
    }
}

fn default_read_max_connections() -> u32 {
    // 读库连接数通常可以更多，因为读操作更频繁
    match std::env::var("APP_ENV").as_deref() {
        Ok("production") => 100,
        _ => 20,
    }
}

/// Redis 配置
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: Secret<String>,
}

/// Kafka 配置
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
}

/// ClickHouse 配置
#[derive(Debug, Clone, Deserialize)]
pub struct ClickHouseConfig {
    pub url: Secret<String>,
    pub database: String,
}

/// JWT 配置
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: Secret<String>,
    #[serde(default = "default_expires_in")]
    pub expires_in: u64,
    #[serde(default = "default_refresh_expires_in")]
    pub refresh_expires_in: u64,
}

fn default_expires_in() -> u64 {
    3600
}

fn default_refresh_expires_in() -> u64 {
    604800
}

/// 服务器配置
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// 遥测配置
#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    pub otlp_endpoint: Option<String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

/// 邮件配置
#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: Secret<String>,
    pub from_email: String,
    pub from_name: String,
    #[serde(default)]
    pub use_tls: bool,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_timeout_secs() -> u64 {
    30
}

/// 密码重置配置
#[derive(Debug, Clone, Deserialize)]
pub struct PasswordResetConfig {
    #[serde(default = "default_token_expires_minutes")]
    pub token_expires_minutes: i64,
    #[serde(default = "default_max_requests_per_hour")]
    pub max_requests_per_hour: u32,
    pub reset_link_base_url: String,
}

fn default_token_expires_minutes() -> i64 {
    15
}

fn default_max_requests_per_hour() -> u32 {
    3
}

/// WebAuthn 配置
#[derive(Debug, Clone, Deserialize)]
pub struct WebAuthnConfig {
    pub rp_id: String,
    pub rp_origin: String,
}

/// 应用配置
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub app_env: String,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub kafka: Option<KafkaConfig>,
    pub clickhouse: Option<ClickHouseConfig>,
    pub jwt: JwtConfig,
    pub server: ServerConfig,
    pub telemetry: TelemetryConfig,
    pub email: EmailConfig,
    pub password_reset: PasswordResetConfig,
    pub webauthn: WebAuthnConfig,
}

impl AppConfig {
    /// 从配置文件和环境变量加载配置
    pub fn load(config_dir: &str) -> Result<Self, ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let config: Self = Figment::new()
            .merge(Toml::file(format!("{}/default.toml", config_dir)))
            .merge(Toml::file(format!("{}/{}.toml", config_dir, env)))
            .merge(Env::prefixed("").split("_"))
            .extract()
            .map_err(Box::new)?;

        Ok(config)
    }

    /// 是否为生产环境
    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }

    /// 是否为开发环境
    pub fn is_development(&self) -> bool {
        self.app_env == "development"
    }
}

#[cfg(test)]
mod tests;
