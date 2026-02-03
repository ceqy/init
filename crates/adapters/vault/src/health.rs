//! Health check functionality for Vault

use errors::AppResult;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::client::VaultClient;

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultHealthStatus {
    /// Whether Vault is accessible
    pub accessible: bool,
    
    /// Whether Vault is sealed
    pub sealed: Option<bool>,
    
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
    
    /// Error message if health check failed
    pub error: Option<String>,
}

impl VaultHealthStatus {
    /// Create a healthy status
    pub fn healthy() -> Self {
        Self {
            accessible: true,
            sealed: Some(false),
            response_time_ms: None,
            error: None,
        }
    }

    /// Create an unhealthy status with error
    pub fn unhealthy(error: String) -> Self {
        Self {
            accessible: false,
            sealed: None,
            response_time_ms: None,
            error: Some(error),
        }
    }

    /// Check if Vault is healthy
    pub fn is_healthy(&self) -> bool {
        self.accessible && !self.sealed.unwrap_or(true)
    }
}

/// Perform health check on Vault client
pub async fn check_vault_health(client: &VaultClient) -> VaultHealthStatus {
    debug!("Performing Vault health check");

    let start = std::time::Instant::now();
    
    match client.health_check().await {
        Ok(accessible) => {
            let response_time_ms = start.elapsed().as_millis() as u64;
            debug!("Vault health check passed in {}ms", response_time_ms);
            
            VaultHealthStatus {
                accessible,
                sealed: Some(false),
                response_time_ms: Some(response_time_ms),
                error: None,
            }
        }
        Err(e) => {
            warn!("Vault health check failed: {}", e);
            VaultHealthStatus::unhealthy(e.to_string())
        }
    }
}

/// Perform a quick connectivity check
pub async fn check_vault_connectivity(client: &VaultClient) -> AppResult<bool> {
    client.health_check().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_status() {
        let status = VaultHealthStatus::healthy();
        assert!(status.is_healthy());
        assert!(status.accessible);
        assert_eq!(status.sealed, Some(false));
        assert!(status.error.is_none());
    }

    #[test]
    fn test_unhealthy_status() {
        let status = VaultHealthStatus::unhealthy("Connection failed".to_string());
        assert!(!status.is_healthy());
        assert!(!status.accessible);
        assert!(status.error.is_some());
    }

    #[test]
    fn test_sealed_vault_is_unhealthy() {
        let status = VaultHealthStatus {
            accessible: true,
            sealed: Some(true),
            response_time_ms: Some(100),
            error: None,
        };
        assert!(!status.is_healthy());
    }
}
