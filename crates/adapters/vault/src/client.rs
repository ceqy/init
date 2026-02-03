//! Vault client implementation

use async_trait::async_trait;
use errors::{AppError, AppResult};
use std::collections::HashMap;
use tracing::{debug, info};
use vaultrs::client::{VaultClient as VaultRsClient, VaultClientSettingsBuilder};
use vaultrs::kv2;
use vaultrs_login::engines::approle::AppRoleLogin;
use vaultrs_login::LoginClient;

use crate::config::VaultConfig;
use crate::error::map_vault_error;

/// Vault client for secret management
pub struct VaultClient {
    client: VaultRsClient,
    mount_path: String,
}

impl VaultClient {
    /// Create a new Vault client with AppRole authentication
    pub async fn new(config: VaultConfig) -> AppResult<Self> {
        info!("Connecting to Vault at {}", config.endpoint);

        // Create client settings
        let settings = VaultClientSettingsBuilder::default()
            .address(&config.endpoint)
            .build()
            .map_err(|e| map_vault_error(e, "Failed to build Vault client settings"))?;

        // Create client
        let mut client = VaultRsClient::new(settings)
            .map_err(|e| map_vault_error(e, "Failed to create Vault client"))?;

        // AppRole authentication
        info!("Authenticating with AppRole");
        let login = AppRoleLogin::new(&config.role_id, &config.secret_id);
        client
            .login("approle", &login)
            .await
            .map_err(|e| map_vault_error(e, "AppRole authentication failed"))?;

        info!("Successfully authenticated with Vault");

        Ok(Self {
            client,
            mount_path: config.mount_path,
        })
    }

    /// Get a secret from Vault
    pub async fn get_secret(&self, path: &str) -> AppResult<HashMap<String, String>> {
        debug!("Reading secret from path: {}", path);

        let secret: HashMap<String, String> = kv2::read(&self.client, &self.mount_path, path)
            .await
            .map_err(|e| map_vault_error(e, &format!("Failed to read secret at path: {}", path)))?;

        debug!("Successfully read secret from path: {}", path);
        Ok(secret)
    }

    /// Get a specific field from a secret
    pub async fn get_secret_field(&self, path: &str, field: &str) -> AppResult<String> {
        let secret = self.get_secret(path).await?;
        secret
            .get(field)
            .cloned()
            .ok_or_else(|| AppError::not_found(format!("Field '{}' not found in secret at path: {}", field, path)))
    }

    /// Set a secret in Vault
    pub async fn set_secret(&self, path: &str, data: HashMap<String, String>) -> AppResult<()> {
        debug!("Writing secret to path: {}", path);

        kv2::set(&self.client, &self.mount_path, path, &data)
            .await
            .map_err(|e| map_vault_error(e, &format!("Failed to write secret at path: {}", path)))?;

        debug!("Successfully wrote secret to path: {}", path);
        Ok(())
    }

    /// Delete a secret from Vault
    pub async fn delete_secret(&self, path: &str) -> AppResult<()> {
        debug!("Deleting secret at path: {}", path);

        kv2::delete_latest(&self.client, &self.mount_path, path)
            .await
            .map_err(|e| map_vault_error(e, &format!("Failed to delete secret at path: {}", path)))?;

        debug!("Successfully deleted secret at path: {}", path);
        Ok(())
    }

    /// Check if Vault is accessible
    pub async fn health_check(&self) -> AppResult<bool> {
        // Try to read a non-existent path to verify connectivity
        // A 404 error means Vault is accessible but path doesn't exist (which is fine)
        match self.get_secret("health-check-probe").await {
            Ok(_) => Ok(true),
            Err(AppError::NotFound(_)) => Ok(true), // 404 is fine for health check
            Err(e) => Err(e),
        }
    }
}

/// Secret manager trait for abstraction
#[async_trait]
pub trait SecretManager: Send + Sync {
    /// Get a secret
    async fn get_secret(&self, path: &str) -> AppResult<HashMap<String, String>>;
    
    /// Get a specific field from a secret
    async fn get_secret_field(&self, path: &str, field: &str) -> AppResult<String>;
    
    /// Set a secret
    async fn set_secret(&self, path: &str, data: HashMap<String, String>) -> AppResult<()>;
    
    /// Delete a secret
    async fn delete_secret(&self, path: &str) -> AppResult<()>;
    
    /// Check health
    async fn health_check(&self) -> AppResult<bool>;
}

#[async_trait]
impl SecretManager for VaultClient {
    async fn get_secret(&self, path: &str) -> AppResult<HashMap<String, String>> {
        self.get_secret(path).await
    }

    async fn get_secret_field(&self, path: &str, field: &str) -> AppResult<String> {
        self.get_secret_field(path, field).await
    }

    async fn set_secret(&self, path: &str, data: HashMap<String, String>) -> AppResult<()> {
        self.set_secret(path, data).await
    }

    async fn delete_secret(&self, path: &str) -> AppResult<()> {
        self.delete_secret(path).await
    }

    async fn health_check(&self) -> AppResult<bool> {
        self.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VaultConfigBuilder;

    #[tokio::test]
    #[ignore] // Requires running Vault server
    async fn test_vault_connection() {
        let config = VaultConfigBuilder::new("http://10.0.0.10:10018")
            .with_approle(
                std::env::var("VAULT_ROLE_ID").unwrap(),
                std::env::var("VAULT_SECRET_ID").unwrap(),
            )
            .build();

        let client = VaultClient::new(config).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires running Vault server
    async fn test_health_check() {
        let config = VaultConfigBuilder::new("http://10.0.0.10:10018")
            .with_approle(
                std::env::var("VAULT_ROLE_ID").unwrap(),
                std::env::var("VAULT_SECRET_ID").unwrap(),
            )
            .build();

        let client = VaultClient::new(config).await.unwrap();
        let health = client.health_check().await;
        assert!(health.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires running Vault server
    async fn test_secret_operations() {
        let config = VaultConfigBuilder::new("http://10.0.0.10:10018")
            .with_approle(
                std::env::var("VAULT_ROLE_ID").unwrap(),
                std::env::var("VAULT_SECRET_ID").unwrap(),
            )
            .build();

        let client = VaultClient::new(config).await.unwrap();

        // Write secret
        let mut data = HashMap::new();
        data.insert("username".to_string(), "testuser".to_string());
        data.insert("password".to_string(), "testpass".to_string());

        let write_result = client.set_secret("test/credentials", data).await;
        assert!(write_result.is_ok());

        // Read secret
        let secret = client.get_secret("test/credentials").await.unwrap();
        assert_eq!(secret.get("username").unwrap(), "testuser");

        // Read specific field
        let username = client.get_secret_field("test/credentials", "username").await.unwrap();
        assert_eq!(username, "testuser");

        // Delete secret
        let delete_result = client.delete_secret("test/credentials").await;
        assert!(delete_result.is_ok());
    }
}
