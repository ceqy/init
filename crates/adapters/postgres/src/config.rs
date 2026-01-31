//! PostgreSQL 配置模块
//!
//! 提供完整的 PostgreSQL 配置，包括连接池、SSL、重试等设置

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// SSL 模式
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SslMode {
    /// 禁用 SSL
    Disable,
    /// 允许 SSL（如果服务器支持）
    #[default]
    Prefer,
    /// 要求 SSL
    Require,
    /// 验证 CA 证书
    VerifyCa,
    /// 验证完整证书链
    VerifyFull,
}

impl SslMode {
    /// 转换为 sqlx 的 SSL 模式字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            SslMode::Disable => "disable",
            SslMode::Prefer => "prefer",
            SslMode::Require => "require",
            SslMode::VerifyCa => "verify-ca",
            SslMode::VerifyFull => "verify-full",
        }
    }
}

/// PostgreSQL 配置
#[derive(Debug, Clone)]
pub struct PostgresConfig {
    // 基础配置
    /// 数据库 URL
    pub url: String,
    /// 主机
    pub host: String,
    /// 端口
    pub port: u16,
    /// 数据库名
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: Option<String>,
    /// Schema
    pub schema: Option<String>,

    // SSL 配置
    /// SSL 模式
    pub ssl_mode: SslMode,
    /// CA 证书路径
    pub ssl_ca_cert: Option<String>,
    /// 客户端证书路径
    pub ssl_client_cert: Option<String>,
    /// 客户端密钥路径
    pub ssl_client_key: Option<String>,

    // 连接池配置
    /// 最小连接数
    pub pool_min: u32,
    /// 最大连接数
    pub pool_max: u32,
    /// 连接超时
    pub connect_timeout: Duration,
    /// 空闲超时
    pub idle_timeout: Duration,
    /// 获取连接超时
    pub acquire_timeout: Duration,
    /// 连接最大生命周期
    pub max_lifetime: Option<Duration>,

    // 重试配置
    /// 最大重试次数
    pub retry_max_attempts: u32,
    /// 初始重试延迟
    pub retry_initial_delay: Duration,
    /// 最大重试延迟
    pub retry_max_delay: Duration,

    // 读写分离配置
    /// 只读副本 URL 列表
    pub read_replicas: Vec<String>,

    // 语句配置
    /// 语句缓存大小
    pub statement_cache_size: usize,
    /// 应用名称（用于连接标识）
    pub application_name: Option<String>,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: None,
            schema: None,
            ssl_mode: SslMode::default(),
            ssl_ca_cert: None,
            ssl_client_cert: None,
            ssl_client_key: None,
            pool_min: 1,
            pool_max: 10,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            acquire_timeout: Duration::from_secs(30),
            max_lifetime: Some(Duration::from_secs(1800)),
            retry_max_attempts: 3,
            retry_initial_delay: Duration::from_millis(100),
            retry_max_delay: Duration::from_secs(5),
            read_replicas: Vec::new(),
            statement_cache_size: 100,
            application_name: None,
        }
    }
}

impl PostgresConfig {
    /// 从 URL 创建配置
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// 从组件创建配置
    pub fn from_components(
        host: impl Into<String>,
        port: u16,
        database: impl Into<String>,
        username: impl Into<String>,
    ) -> Self {
        let host = host.into();
        let database = database.into();
        let username = username.into();
        let url = format!("postgres://{}@{}:{}/{}", username, host, port, database);

        Self {
            url,
            host,
            port,
            database,
            username,
            ..Default::default()
        }
    }

    /// 设置密码
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self.rebuild_url();
        self
    }

    /// 设置 Schema
    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    /// 设置 SSL 模式
    pub fn with_ssl_mode(mut self, mode: SslMode) -> Self {
        self.ssl_mode = mode;
        self
    }

    /// 设置 SSL 证书
    pub fn with_ssl_certs(
        mut self,
        ca_cert: Option<String>,
        client_cert: Option<String>,
        client_key: Option<String>,
    ) -> Self {
        self.ssl_ca_cert = ca_cert;
        self.ssl_client_cert = client_cert;
        self.ssl_client_key = client_key;
        self
    }

    /// 设置连接池配置
    pub fn with_pool(mut self, min: u32, max: u32) -> Self {
        self.pool_min = min;
        self.pool_max = max;
        self
    }

    /// 设置连接超时
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// 设置空闲超时
    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// 设置获取连接超时
    pub fn with_acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }

    /// 设置连接最大生命周期
    pub fn with_max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime = Some(lifetime);
        self
    }

    /// 设置重试配置
    pub fn with_retry(
        mut self,
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        self.retry_max_attempts = max_attempts;
        self.retry_initial_delay = initial_delay;
        self.retry_max_delay = max_delay;
        self
    }

    /// 添加只读副本
    pub fn with_read_replica(mut self, url: impl Into<String>) -> Self {
        self.read_replicas.push(url.into());
        self
    }

    /// 设置只读副本列表
    pub fn with_read_replicas(mut self, urls: Vec<String>) -> Self {
        self.read_replicas = urls;
        self
    }

    /// 设置语句缓存大小
    pub fn with_statement_cache_size(mut self, size: usize) -> Self {
        self.statement_cache_size = size;
        self
    }

    /// 设置应用名称
    pub fn with_application_name(mut self, name: impl Into<String>) -> Self {
        self.application_name = Some(name.into());
        self
    }

    /// 是否启用了读写分离
    pub fn has_read_replicas(&self) -> bool {
        !self.read_replicas.is_empty()
    }

    /// 重建 URL
    fn rebuild_url(&mut self) {
        let password_part = self
            .password
            .as_ref()
            .map(|p| format!(":{}@", p))
            .unwrap_or_else(|| "@".to_string());

        self.url = format!(
            "postgres://{}{}{}:{}/{}",
            self.username, password_part, self.host, self.port, self.database
        );
    }

    /// 获取带 SSL 参数的连接 URL
    pub fn connection_url(&self) -> String {
        let mut url = self.url.clone();
        let mut params = Vec::new();

        params.push(format!("sslmode={}", self.ssl_mode.as_str()));

        if let Some(ref app_name) = self.application_name {
            params.push(format!("application_name={}", app_name));
        }

        if let Some(ref schema) = self.schema {
            params.push(format!("options=--search_path%3D{}", schema));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        url
    }
}

/// 重试配置（使用 cuba_common::RetryConfig）
pub type RetryConfig = cuba_common::RetryConfig;

/// 从 PostgresConfig 创建重试配置
pub fn retry_config_from_postgres_config(config: &PostgresConfig) -> RetryConfig {
    RetryConfig::new(
        config.retry_max_attempts,
        config.retry_initial_delay,
        config.retry_max_delay,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PostgresConfig::default();
        assert_eq!(config.pool_min, 1);
        assert_eq!(config.pool_max, 10);
        assert_eq!(config.ssl_mode, SslMode::Prefer);
        assert_eq!(config.port, 5432);
    }

    #[test]
    fn test_config_from_url() {
        let config = PostgresConfig::new("postgres://user:pass@localhost:5432/mydb")
            .with_pool(2, 20)
            .with_application_name("myapp");

        assert_eq!(config.pool_min, 2);
        assert_eq!(config.pool_max, 20);
        assert_eq!(config.application_name, Some("myapp".to_string()));
    }

    #[test]
    fn test_config_from_components() {
        let config =
            PostgresConfig::from_components("db.example.com", 5433, "production", "admin")
                .with_password("secret")
                .with_schema("app");

        assert_eq!(config.host, "db.example.com");
        assert_eq!(config.port, 5433);
        assert_eq!(config.database, "production");
        assert_eq!(config.username, "admin");
        assert_eq!(config.password, Some("secret".to_string()));
        assert_eq!(config.schema, Some("app".to_string()));
    }

    #[test]
    fn test_ssl_mode() {
        assert_eq!(SslMode::Disable.as_str(), "disable");
        assert_eq!(SslMode::Prefer.as_str(), "prefer");
        assert_eq!(SslMode::Require.as_str(), "require");
        assert_eq!(SslMode::VerifyCa.as_str(), "verify-ca");
        assert_eq!(SslMode::VerifyFull.as_str(), "verify-full");
    }

    #[test]
    fn test_connection_url() {
        let config = PostgresConfig::new("postgres://user@localhost:5432/mydb")
            .with_ssl_mode(SslMode::Require)
            .with_application_name("test_app")
            .with_schema("public");

        let url = config.connection_url();
        assert!(url.contains("sslmode=require"));
        assert!(url.contains("application_name=test_app"));
        assert!(url.contains("options=--search_path%3Dpublic"));
    }

    #[test]
    fn test_read_replicas() {
        let config = PostgresConfig::new("postgres://localhost/primary")
            .with_read_replica("postgres://replica1/db")
            .with_read_replica("postgres://replica2/db");

        assert!(config.has_read_replicas());
        assert_eq!(config.read_replicas.len(), 2);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig::new(
            5,
            Duration::from_millis(100),
            Duration::from_secs(5),
        );

        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(config.delay_for_attempt(10), Duration::from_secs(5));
    }
}
