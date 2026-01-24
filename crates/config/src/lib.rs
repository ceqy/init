//! cuba-config - 配置加载库

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load config: {0}")]
    Load(#[from] figment::Error),
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_max_connections() -> u32 {
    10
}

/// Redis 配置
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

/// Kafka 配置
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
}

/// ClickHouse 配置
#[derive(Debug, Clone, Deserialize)]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
}

/// JWT 配置
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
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
}

impl AppConfig {
    /// 从配置文件和环境变量加载配置
    pub fn load(config_dir: &str) -> Result<Self, ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let config: Self = Figment::new()
            .merge(Toml::file(format!("{}/default.toml", config_dir)))
            .merge(Toml::file(format!("{}/{}.toml", config_dir, env)))
            .merge(Env::prefixed("").split("_"))
            .extract()?;

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
