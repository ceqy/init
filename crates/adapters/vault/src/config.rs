//! Vault configuration

use serde::Deserialize;

/// Vault client configuration
#[derive(Debug, Clone, Deserialize)]
pub struct VaultConfig {
    /// Vault server endpoint
    pub endpoint: String,
    
    /// AppRole role ID for authentication
    pub role_id: String,
    
    /// AppRole secret ID for authentication
    pub secret_id: String,
    
    /// KV secrets engine mount path
    #[serde(default = "default_mount_path")]
    pub mount_path: String,
    
    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,
    
    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,
    
    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_mount_path() -> String {
    "secret".to_string()
}

fn default_connection_timeout() -> u64 {
    10
}

fn default_request_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8200".to_string(),
            role_id: String::new(),
            secret_id: String::new(),
            mount_path: default_mount_path(),
            connection_timeout_secs: default_connection_timeout(),
            request_timeout_secs: default_request_timeout(),
            max_retries: default_max_retries(),
        }
    }
}

/// Builder for VaultConfig
pub struct VaultConfigBuilder {
    config: VaultConfig,
}

impl VaultConfigBuilder {
    /// Create a new builder with endpoint
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            config: VaultConfig {
                endpoint: endpoint.into(),
                ..Default::default()
            },
        }
    }
    
    /// Set AppRole credentials
    pub fn with_approle(mut self, role_id: impl Into<String>, secret_id: impl Into<String>) -> Self {
        self.config.role_id = role_id.into();
        self.config.secret_id = secret_id.into();
        self
    }
    
    /// Set mount path
    pub fn with_mount_path(mut self, mount_path: impl Into<String>) -> Self {
        self.config.mount_path = mount_path.into();
        self
    }
    
    /// Set connection timeout
    pub fn with_connection_timeout(mut self, timeout_secs: u64) -> Self {
        self.config.connection_timeout_secs = timeout_secs;
        self
    }
    
    /// Set request timeout
    pub fn with_request_timeout(mut self, timeout_secs: u64) -> Self {
        self.config.request_timeout_secs = timeout_secs;
        self
    }
    
    /// Set max retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> VaultConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VaultConfig::default();
        assert_eq!(config.endpoint, "http://localhost:8200");
        assert_eq!(config.mount_path, "secret");
        assert_eq!(config.connection_timeout_secs, 10);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_builder() {
        let config = VaultConfigBuilder::new("http://vault:8200")
            .with_approle("role123", "secret456")
            .with_mount_path("kv")
            .with_connection_timeout(20)
            .with_max_retries(5)
            .build();

        assert_eq!(config.endpoint, "http://vault:8200");
        assert_eq!(config.role_id, "role123");
        assert_eq!(config.secret_id, "secret456");
        assert_eq!(config.mount_path, "kv");
        assert_eq!(config.connection_timeout_secs, 20);
        assert_eq!(config.max_retries, 5);
    }
}
