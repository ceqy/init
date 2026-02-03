//! config - 配置加载库

use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

use secrecy::Secret;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load config: {0}")]
    Load(#[from] Box<figment::Error>),
    #[error("Vault error: {0}")]
    Vault(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
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
    pub bootstrap_servers: String,
    pub host: String,
    pub port: u16,
    pub security_protocol: Option<String>,
    pub sasl_mechanism: Option<String>,
    pub username: Option<String>,
    pub password: Option<Secret<String>>,
}

/// Etcd 配置
#[derive(Debug, Clone, Deserialize)]
pub struct EtcdConfig {
    pub url: Secret<String>,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<Secret<String>>,
}

/// ClickHouse 配置
#[derive(Debug, Clone, Deserialize)]
pub struct ClickHouseConfig {
    pub url: Secret<String>,
    pub database: String,
    pub user: Option<String>,
    pub password: Option<Secret<String>>,

    // 连接池配置
    #[serde(default = "default_ch_pool_min")]
    pub pool_min: u32,
    #[serde(default = "default_ch_pool_max")]
    pub pool_max: u32,
    #[serde(default = "default_ch_connection_timeout_secs")]
    pub connection_timeout_secs: u64,
    #[serde(default = "default_ch_idle_timeout_secs")]
    pub idle_timeout_secs: u64,

    // 重试配置
    #[serde(default = "default_ch_retry_max_attempts")]
    pub retry_max_attempts: u32,
    #[serde(default = "default_ch_retry_initial_delay_ms")]
    pub retry_initial_delay_ms: u64,
    #[serde(default = "default_ch_retry_max_delay_ms")]
    pub retry_max_delay_ms: u64,

    // 批量写入配置
    #[serde(default = "default_ch_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_ch_batch_timeout_secs")]
    pub batch_timeout_secs: u64,

    // 压缩配置
    #[serde(default = "default_ch_compression")]
    pub compression: String,

    // 集群配置（可选）
    pub cluster_name: Option<String>,
    #[serde(default)]
    pub replicas: Vec<ClickHouseReplicaConfig>,
}

/// ClickHouse 副本配置
#[derive(Debug, Clone, Deserialize)]
pub struct ClickHouseReplicaConfig {
    pub url: String,
    #[serde(default = "default_replica_weight")]
    pub weight: u32,
}

fn default_ch_pool_min() -> u32 {
    1
}

fn default_ch_pool_max() -> u32 {
    10
}

fn default_ch_connection_timeout_secs() -> u64 {
    30
}

fn default_ch_idle_timeout_secs() -> u64 {
    600
}

fn default_ch_retry_max_attempts() -> u32 {
    3
}

fn default_ch_retry_initial_delay_ms() -> u64 {
    100
}

fn default_ch_retry_max_delay_ms() -> u64 {
    10000
}

fn default_ch_batch_size() -> usize {
    10000
}

fn default_ch_batch_timeout_secs() -> u64 {
    5
}

fn default_ch_compression() -> String {
    "lz4".to_string()
}

fn default_replica_weight() -> u32 {
    1
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

/// MinIO 配置
#[derive(Debug, Clone, Deserialize)]
pub struct MinioConfig {
    pub endpoint: String,
    pub console_url: Option<String>,
    pub host: String,
    pub api_port: u16,
    pub console_port: Option<u16>,
    pub access_key: Secret<String>,
    pub secret_key: Secret<String>,
    pub region: Option<String>,
}

/// Elasticsearch 配置
#[derive(Debug, Clone, Deserialize)]
pub struct ElasticsearchConfig {
    pub url: String,
    pub username: String,
    pub password: Secret<String>,
}

/// Grafana 配置
#[derive(Debug, Clone, Deserialize)]
pub struct GrafanaConfig {
    pub url: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
}

/// Prometheus 配置
#[derive(Debug, Clone, Deserialize)]
pub struct PrometheusConfig {
    pub url: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<Secret<String>>,
}

/// Alertmanager 配置
#[derive(Debug, Clone, Deserialize)]
pub struct AlertmanagerConfig {
    pub url: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<Secret<String>>,
}

/// Node Exporter 配置
#[derive(Debug, Clone, Deserialize)]
pub struct NodeExporterConfig {
    pub url: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<Secret<String>>,
}

/// Loki 配置
#[derive(Debug, Clone, Deserialize)]
pub struct LokiConfig {
    pub url: String,
    pub host: String,
    pub port: u16,
}

/// Jaeger 配置
#[derive(Debug, Clone, Deserialize)]
pub struct JaegerConfig {
    pub url: String,
    pub host: String,
    pub ui_port: u16,
    pub otlp_grpc_port: u16,
    pub otlp_http_port: u16,
}

/// Promtail 配置
#[derive(Debug, Clone, Deserialize)]
pub struct PromtailConfig {
    pub url: String,
    pub host: String,
    pub port: u16,
}

/// RabbitMQ 配置
#[derive(Debug, Clone, Deserialize)]
pub struct RabbitMQConfig {
    pub url: Secret<String>,
    pub management_url: Option<String>,
    pub host: String,
    pub amqp_port: u16,
    pub management_port: Option<u16>,
    pub username: String,
    pub password: Secret<String>,
    pub connection_string: Option<Secret<String>>,
}

/// SSH 配置
#[derive(Debug, Clone, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
}

/// Cockpit 配置
#[derive(Debug, Clone, Deserialize)]
pub struct CockpitConfig {
    pub url: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
}

/// 消息队列聚合配置
#[derive(Debug, Clone, Deserialize)]
pub struct MqConfig {
    pub kafka: Option<KafkaConfig>,
    pub rabbitmq: Option<RabbitMQConfig>,
}

/// 监控服务聚合配置
#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringConfig {
    pub grafana: Option<GrafanaConfig>,
    pub prometheus: Option<PrometheusConfig>,
    pub alertmanager: Option<AlertmanagerConfig>,
    pub node_exporter: Option<NodeExporterConfig>,
    pub loki: Option<LokiConfig>,
    pub jaeger: Option<JaegerConfig>,
    pub promtail: Option<PromtailConfig>,
}

/// 系统服务聚合配置
#[derive(Debug, Clone, Deserialize)]
pub struct SystemConfig {
    pub ssh: Option<SshConfig>,
    pub cockpit: Option<CockpitConfig>,
}

/// 应用配置
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub app_env: String,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub etcd: Option<EtcdConfig>,
    pub clickhouse: Option<ClickHouseConfig>,
    pub jwt: JwtConfig,
    pub server: ServerConfig,
    pub telemetry: TelemetryConfig,
    pub email: EmailConfig,
    pub password_reset: PasswordResetConfig,
    pub webauthn: WebAuthnConfig,
    pub minio: Option<MinioConfig>,
    pub elasticsearch: Option<ElasticsearchConfig>,
    pub mq: Option<MqConfig>,
    pub monitoring: Option<MonitoringConfig>,
    pub system: Option<SystemConfig>,
}

impl AppConfig {
    /// 从配置文件和环境变量加载配置
    pub async fn load(config_dir: &str) -> Result<Self, ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        // 1. 加载引导配置 (Vault 凭证)
        let vault_role_id = std::env::var("VAULT_ROLE_ID").ok();
        let vault_secret_id = std::env::var("VAULT_SECRET_ID").ok();
        let vault_addr = std::env::var("VAULT_ADDR").ok();

        let mut vault_secrets = serde_json::json!({});

        if let (Some(role_id), Some(secret_id), Some(addr)) =
            (vault_role_id, vault_secret_id, vault_addr)
        {
            // 2. 准备拉取路径
            let mut paths = HashMap::new();
            // 核心服务
            paths.insert(
                "database",
                std::env::var("VAULT_DB_PATH").unwrap_or_default(),
            );
            paths.insert(
                "redis",
                std::env::var("VAULT_REDIS_PATH").unwrap_or_default(),
            );
            paths.insert("etcd", std::env::var("VAULT_ETCD_PATH").unwrap_or_default());
            paths.insert(
                "clickhouse",
                std::env::var("VAULT_CLICKHOUSE_PATH").unwrap_or_default(),
            );

            // 存储与搜索
            paths.insert(
                "minio",
                std::env::var("VAULT_MINIO_PATH").unwrap_or_default(),
            );
            paths.insert(
                "elasticsearch",
                std::env::var("VAULT_ES_PATH").unwrap_or_default(),
            );

            // 消息队列 (聚合)
            paths.insert(
                "mq.kafka",
                std::env::var("VAULT_KAFKA_PATH").unwrap_or_default(),
            );
            paths.insert(
                "mq.rabbitmq",
                std::env::var("VAULT_RABBITMQ_PATH").unwrap_or_default(),
            );

            // 监控 (聚合)
            paths.insert(
                "monitoring.grafana",
                std::env::var("VAULT_GRAFANA_PATH").unwrap_or_default(),
            );
            paths.insert(
                "monitoring.prometheus",
                std::env::var("VAULT_PROMETHEUS_PATH").unwrap_or_default(),
            );
            paths.insert(
                "monitoring.alertmanager",
                std::env::var("VAULT_ALERTMANAGER_PATH").unwrap_or_default(),
            );
            paths.insert(
                "monitoring.node_exporter",
                std::env::var("VAULT_NODE_EXPORTER_PATH").unwrap_or_default(),
            );
            paths.insert(
                "monitoring.loki",
                std::env::var("VAULT_LOKI_PATH").unwrap_or_default(),
            );
            paths.insert(
                "monitoring.jaeger",
                std::env::var("VAULT_JAEGER_PATH").unwrap_or_default(),
            );
            paths.insert(
                "monitoring.promtail",
                std::env::var("VAULT_PROMTAIL_PATH").unwrap_or_default(),
            );

            // 系统服务 (聚合)
            paths.insert(
                "system.ssh",
                std::env::var("VAULT_SSH_PATH").unwrap_or_default(),
            );
            paths.insert(
                "system.cockpit",
                std::env::var("VAULT_COCKPIT_PATH").unwrap_or_default(),
            );

            // 3. 执行 Vault 登录和拉取逻辑
            vault_secrets = fetch_vault_secrets(&addr, &role_id, &secret_id, paths).await?;
        }

        // 3. 合并所有配置源
        let mut figment = Figment::new()
            .merge(Toml::file(format!("{}/default.toml", config_dir)))
            .merge(Toml::file(format!("{}/{}.toml", config_dir, env)))
            .merge(Env::prefixed("").split("_"));

        if !vault_secrets.is_null()
            && vault_secrets
                .as_object()
                .map(|o| !o.is_empty())
                .unwrap_or(false)
        {
            figment = figment.merge(Serialized::globals(vault_secrets));
        }

        let config: Self = figment.extract().map_err(Box::new)?;

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
/// 从 Vault 获取秘密并解析为 Figment 可用的 JSON 对象
async fn fetch_vault_secrets(
    addr: &str,
    role_id: &str,
    secret_id: &str,
    paths: HashMap<&str, String>,
) -> Result<serde_json::Value, ConfigError> {
    let client = reqwest::Client::builder().no_proxy().build()?;

    let login_url = format!("{}/v1/auth/approle/login", addr);
    let login_resp_raw = client
        .post(&login_url)
        .json(&serde_json::json!({ "role_id": role_id, "secret_id": secret_id }))
        .send()
        .await?;

    let login_text = login_resp_raw.text().await?;
    let login_resp: serde_json::Value = serde_json::from_str(&login_text)
        .map_err(|e| ConfigError::Vault(format!("Failed to parse login JSON: {}", e)))?;

    let token = login_resp["auth"]["client_token"]
        .as_str()
        .ok_or_else(|| ConfigError::Vault("Failed to get client token".into()))?;

    let mut root = serde_json::json!({});

    for (prefix, path) in paths {
        if path.is_empty() {
            continue;
        }

        let secret_url = format!("{}/v1/{}", addr, path);
        let resp: serde_json::Value = client
            .get(&secret_url)
            .header("X-Vault-Token", token)
            .send()
            .await?
            .json()
            .await?;

        if let Some(data) = resp["data"]["data"].as_object() {
            let parts: Vec<&str> = prefix.split('.').collect();
            let mut current = root.as_object_mut().unwrap();

            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    let section = current
                        .entry(part.to_string())
                        .or_insert(serde_json::json!({}))
                        .as_object_mut()
                        .ok_or_else(|| ConfigError::Vault(format!("Path conflict at {}", part)))?;

                    for (key, val) in data {
                        // 智能类型转换：Vault KV 通常将所有内容存为字符串
                        // 如果字符串看起来像数字，我们尝试将其转为 JSON Number，以便 Figment 正确解析为 u16/u32
                        let final_val = if let Some(s) = val.as_str() {
                            if let Ok(n) = s.parse::<i64>() {
                                serde_json::json!(n)
                            } else if let Ok(f) = s.parse::<f64>() {
                                serde_json::json!(f)
                            } else if s == "true" {
                                serde_json::json!(true)
                            } else if s == "false" {
                                serde_json::json!(false)
                            } else {
                                val.clone()
                            }
                        } else {
                            val.clone()
                        };
                        section.insert(key.clone(), final_val);
                    }
                } else {
                    let next = current
                        .entry(part.to_string())
                        .or_insert(serde_json::json!({}))
                        .as_object_mut()
                        .ok_or_else(|| ConfigError::Vault(format!("Path conflict at {}", part)))?;
                    current = next;
                }
            }
        }
    }

    Ok(root)
}

#[cfg(test)]
mod tests;
