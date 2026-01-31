//! Kafka 配置模块
//!
//! 提供统一的 Kafka 配置管理

use std::collections::HashMap;
use std::time::Duration;

/// Kafka 安全协议
#[derive(Debug, Clone, Default)]
pub enum SecurityProtocol {
    /// 明文（默认）
    #[default]
    Plaintext,
    /// SSL
    Ssl,
    /// SASL 明文
    SaslPlaintext,
    /// SASL SSL
    SaslSsl,
}

impl SecurityProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityProtocol::Plaintext => "plaintext",
            SecurityProtocol::Ssl => "ssl",
            SecurityProtocol::SaslPlaintext => "sasl_plaintext",
            SecurityProtocol::SaslSsl => "sasl_ssl",
        }
    }
}

/// SASL 认证机制
#[derive(Debug, Clone)]
pub enum SaslMechanism {
    Plain,
    ScramSha256,
    ScramSha512,
    OAuthBearer,
}

impl SaslMechanism {
    pub fn as_str(&self) -> &'static str {
        match self {
            SaslMechanism::Plain => "PLAIN",
            SaslMechanism::ScramSha256 => "SCRAM-SHA-256",
            SaslMechanism::ScramSha512 => "SCRAM-SHA-512",
            SaslMechanism::OAuthBearer => "OAUTHBEARER",
        }
    }
}

/// SASL 配置
#[derive(Debug, Clone)]
pub struct SaslConfig {
    /// 认证机制
    pub mechanism: SaslMechanism,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
}

impl SaslConfig {
    pub fn plain(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            mechanism: SaslMechanism::Plain,
            username: username.into(),
            password: password.into(),
        }
    }

    pub fn scram_sha256(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            mechanism: SaslMechanism::ScramSha256,
            username: username.into(),
            password: password.into(),
        }
    }

    pub fn scram_sha512(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            mechanism: SaslMechanism::ScramSha512,
            username: username.into(),
            password: password.into(),
        }
    }
}

/// SSL 配置
#[derive(Debug, Clone, Default)]
pub struct SslConfig {
    /// CA 证书路径
    pub ca_location: Option<String>,
    /// 客户端证书路径
    pub certificate_location: Option<String>,
    /// 客户端私钥路径
    pub key_location: Option<String>,
    /// 私钥密码
    pub key_password: Option<String>,
    /// 是否验证服务器证书
    pub enable_verification: bool,
}

impl SslConfig {
    pub fn new() -> Self {
        Self {
            enable_verification: true,
            ..Default::default()
        }
    }

    pub fn with_ca(mut self, ca_location: impl Into<String>) -> Self {
        self.ca_location = Some(ca_location.into());
        self
    }

    pub fn with_client_cert(
        mut self,
        cert_location: impl Into<String>,
        key_location: impl Into<String>,
    ) -> Self {
        self.certificate_location = Some(cert_location.into());
        self.key_location = Some(key_location.into());
        self
    }

    pub fn with_key_password(mut self, password: impl Into<String>) -> Self {
        self.key_password = Some(password.into());
        self
    }

    pub fn without_verification(mut self) -> Self {
        self.enable_verification = false;
        self
    }
}

/// 压缩类型
#[derive(Debug, Clone, Default)]
pub enum CompressionType {
    #[default]
    None,
    Gzip,
    Snappy,
    Lz4,
    Zstd,
}

impl CompressionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompressionType::None => "none",
            CompressionType::Gzip => "gzip",
            CompressionType::Snappy => "snappy",
            CompressionType::Lz4 => "lz4",
            CompressionType::Zstd => "zstd",
        }
    }
}

/// Kafka 基础配置
#[derive(Debug, Clone)]
pub struct KafkaConfig {
    /// Broker 地址列表
    pub brokers: String,
    /// 客户端 ID
    pub client_id: Option<String>,
    /// 安全协议
    pub security_protocol: SecurityProtocol,
    /// SASL 配置
    pub sasl: Option<SaslConfig>,
    /// SSL 配置
    pub ssl: Option<SslConfig>,
    /// 额外配置
    pub extra: HashMap<String, String>,
}

impl KafkaConfig {
    pub fn new(brokers: impl Into<String>) -> Self {
        Self {
            brokers: brokers.into(),
            client_id: None,
            security_protocol: SecurityProtocol::default(),
            sasl: None,
            ssl: None,
            extra: HashMap::new(),
        }
    }

    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    pub fn with_security_protocol(mut self, protocol: SecurityProtocol) -> Self {
        self.security_protocol = protocol;
        self
    }

    pub fn with_sasl(mut self, sasl: SaslConfig) -> Self {
        self.sasl = Some(sasl);
        if matches!(self.security_protocol, SecurityProtocol::Plaintext) {
            self.security_protocol = SecurityProtocol::SaslPlaintext;
        }
        self
    }

    pub fn with_ssl(mut self, ssl: SslConfig) -> Self {
        self.ssl = Some(ssl);
        if matches!(self.security_protocol, SecurityProtocol::Plaintext) {
            self.security_protocol = SecurityProtocol::Ssl;
        }
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }

    /// 转换为 rdkafka ClientConfig 的配置项
    pub fn to_client_config_entries(&self) -> Vec<(String, String)> {
        let mut entries = vec![
            ("bootstrap.servers".to_string(), self.brokers.clone()),
            (
                "security.protocol".to_string(),
                self.security_protocol.as_str().to_string(),
            ),
        ];

        if let Some(client_id) = &self.client_id {
            entries.push(("client.id".to_string(), client_id.clone()));
        }

        if let Some(sasl) = &self.sasl {
            entries.push((
                "sasl.mechanism".to_string(),
                sasl.mechanism.as_str().to_string(),
            ));
            entries.push(("sasl.username".to_string(), sasl.username.clone()));
            entries.push(("sasl.password".to_string(), sasl.password.clone()));
        }

        if let Some(ssl) = &self.ssl {
            if let Some(ca) = &ssl.ca_location {
                entries.push(("ssl.ca.location".to_string(), ca.clone()));
            }
            if let Some(cert) = &ssl.certificate_location {
                entries.push(("ssl.certificate.location".to_string(), cert.clone()));
            }
            if let Some(key) = &ssl.key_location {
                entries.push(("ssl.key.location".to_string(), key.clone()));
            }
            if let Some(password) = &ssl.key_password {
                entries.push(("ssl.key.password".to_string(), password.clone()));
            }
            if !ssl.enable_verification {
                entries.push((
                    "ssl.endpoint.identification.algorithm".to_string(),
                    "none".to_string(),
                ));
            }
        }

        for (key, value) in &self.extra {
            entries.push((key.clone(), value.clone()));
        }

        entries
    }
}

/// Producer 配置
#[derive(Debug, Clone)]
pub struct ProducerConfig {
    /// 基础配置
    pub base: KafkaConfig,
    /// 压缩类型
    pub compression: CompressionType,
    /// 批量大小（字节）
    pub batch_size: usize,
    /// 延迟发送时间（用于批量）
    pub linger_ms: u64,
    /// 确认模式：0=不等待，1=leader确认，-1=所有副本确认
    pub acks: i32,
    /// 重试次数
    pub retries: u32,
    /// 请求超时
    pub request_timeout: Duration,
    /// 幂等性
    pub enable_idempotence: bool,
}

impl ProducerConfig {
    pub fn new(brokers: impl Into<String>) -> Self {
        Self {
            base: KafkaConfig::new(brokers),
            compression: CompressionType::default(),
            batch_size: 16384,
            linger_ms: 5,
            acks: -1,
            retries: 3,
            request_timeout: Duration::from_secs(30),
            enable_idempotence: false,
        }
    }

    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.base = self.base.with_client_id(client_id);
        self
    }

    pub fn with_sasl(mut self, sasl: SaslConfig) -> Self {
        self.base = self.base.with_sasl(sasl);
        self
    }

    pub fn with_ssl(mut self, ssl: SslConfig) -> Self {
        self.base = self.base.with_ssl(ssl);
        self
    }

    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = compression;
        self
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    pub fn with_linger_ms(mut self, ms: u64) -> Self {
        self.linger_ms = ms;
        self
    }

    pub fn with_acks(mut self, acks: i32) -> Self {
        self.acks = acks;
        self
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    pub fn with_idempotence(mut self) -> Self {
        self.enable_idempotence = true;
        self.acks = -1; // 幂等性需要 acks=-1
        self
    }

    /// 转换为 rdkafka ClientConfig 的配置项
    pub fn to_client_config_entries(&self) -> Vec<(String, String)> {
        let mut entries = self.base.to_client_config_entries();

        entries.push((
            "compression.type".to_string(),
            self.compression.as_str().to_string(),
        ));
        entries.push(("batch.size".to_string(), self.batch_size.to_string()));
        entries.push(("linger.ms".to_string(), self.linger_ms.to_string()));
        entries.push(("acks".to_string(), self.acks.to_string()));
        entries.push(("retries".to_string(), self.retries.to_string()));
        entries.push((
            "request.timeout.ms".to_string(),
            self.request_timeout.as_millis().to_string(),
        ));

        if self.enable_idempotence {
            entries.push(("enable.idempotence".to_string(), "true".to_string()));
        }

        entries
    }
}

/// Consumer 配置
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// 基础配置
    pub base: KafkaConfig,
    /// 消费者组 ID
    pub group_id: String,
    /// 订阅的 topics
    pub topics: Vec<String>,
    /// 自动提交
    pub enable_auto_commit: bool,
    /// 自动提交间隔
    pub auto_commit_interval: Duration,
    /// 自动偏移重置策略
    pub auto_offset_reset: AutoOffsetReset,
    /// 最大拉取字节数
    pub max_fetch_bytes: usize,
    /// 会话超时
    pub session_timeout: Duration,
    /// 心跳间隔
    pub heartbeat_interval: Duration,
    /// 最大重试次数
    pub max_retries: u32,
    /// 是否启用 DLQ
    pub enable_dlq: bool,
    /// DLQ topic 后缀
    pub dlq_suffix: String,
}

/// 自动偏移重置策略
#[derive(Debug, Clone, Default)]
pub enum AutoOffsetReset {
    #[default]
    Earliest,
    Latest,
    None,
}

impl AutoOffsetReset {
    pub fn as_str(&self) -> &'static str {
        match self {
            AutoOffsetReset::Earliest => "earliest",
            AutoOffsetReset::Latest => "latest",
            AutoOffsetReset::None => "none",
        }
    }
}

impl ConsumerConfig {
    pub fn new(brokers: impl Into<String>, group_id: impl Into<String>) -> Self {
        Self {
            base: KafkaConfig::new(brokers),
            group_id: group_id.into(),
            topics: Vec::new(),
            enable_auto_commit: false,
            auto_commit_interval: Duration::from_secs(5),
            auto_offset_reset: AutoOffsetReset::default(),
            max_fetch_bytes: 52428800, // 50MB
            session_timeout: Duration::from_secs(45),
            heartbeat_interval: Duration::from_secs(3),
            max_retries: 3,
            enable_dlq: true,
            dlq_suffix: ".dlq".to_string(),
        }
    }

    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.base = self.base.with_client_id(client_id);
        self
    }

    pub fn with_sasl(mut self, sasl: SaslConfig) -> Self {
        self.base = self.base.with_sasl(sasl);
        self
    }

    pub fn with_ssl(mut self, ssl: SslConfig) -> Self {
        self.base = self.base.with_ssl(ssl);
        self
    }

    pub fn with_topics(mut self, topics: Vec<String>) -> Self {
        self.topics = topics;
        self
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topics.push(topic.into());
        self
    }

    pub fn with_auto_commit(mut self, enable: bool) -> Self {
        self.enable_auto_commit = enable;
        self
    }

    pub fn with_auto_offset_reset(mut self, reset: AutoOffsetReset) -> Self {
        self.auto_offset_reset = reset;
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn with_dlq(mut self, enable: bool) -> Self {
        self.enable_dlq = enable;
        self
    }

    pub fn with_dlq_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.dlq_suffix = suffix.into();
        self
    }

    /// 转换为 rdkafka ClientConfig 的配置项
    pub fn to_client_config_entries(&self) -> Vec<(String, String)> {
        let mut entries = self.base.to_client_config_entries();

        entries.push(("group.id".to_string(), self.group_id.clone()));
        entries.push((
            "enable.auto.commit".to_string(),
            self.enable_auto_commit.to_string(),
        ));
        entries.push((
            "auto.commit.interval.ms".to_string(),
            self.auto_commit_interval.as_millis().to_string(),
        ));
        entries.push((
            "auto.offset.reset".to_string(),
            self.auto_offset_reset.as_str().to_string(),
        ));
        entries.push((
            "fetch.message.max.bytes".to_string(),
            self.max_fetch_bytes.to_string(),
        ));
        entries.push((
            "session.timeout.ms".to_string(),
            self.session_timeout.as_millis().to_string(),
        ));
        entries.push((
            "heartbeat.interval.ms".to_string(),
            self.heartbeat_interval.as_millis().to_string(),
        ));

        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kafka_config() {
        let config = KafkaConfig::new("localhost:9092")
            .with_client_id("test-client")
            .with_sasl(SaslConfig::plain("user", "pass"));

        let entries = config.to_client_config_entries();
        assert!(entries
            .iter()
            .any(|(k, v)| k == "bootstrap.servers" && v == "localhost:9092"));
        assert!(entries
            .iter()
            .any(|(k, v)| k == "sasl.mechanism" && v == "PLAIN"));
    }

    #[test]
    fn test_producer_config() {
        let config = ProducerConfig::new("localhost:9092")
            .with_compression(CompressionType::Lz4)
            .with_idempotence();

        let entries = config.to_client_config_entries();
        assert!(entries
            .iter()
            .any(|(k, v)| k == "compression.type" && v == "lz4"));
        assert!(entries
            .iter()
            .any(|(k, v)| k == "enable.idempotence" && v == "true"));
    }

    #[test]
    fn test_consumer_config() {
        let config = ConsumerConfig::new("localhost:9092", "test-group")
            .with_topic("topic1")
            .with_topic("topic2")
            .with_auto_offset_reset(AutoOffsetReset::Latest);

        assert_eq!(config.topics.len(), 2);
        let entries = config.to_client_config_entries();
        assert!(entries
            .iter()
            .any(|(k, v)| k == "group.id" && v == "test-group"));
    }
}
