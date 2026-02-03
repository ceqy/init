//! adapter-vault - HashiCorp Vault adapter
//!
//! Provides unified secret management interface with support for:
//! - AppRole authentication
//! - KV v2 secrets engine
//! - Health checking
//! - Automatic error mapping to AppError

pub mod client;
pub mod config;
pub mod error;
pub mod health;

pub use client::{SecretManager, VaultClient};
pub use config::{VaultConfig, VaultConfigBuilder};
pub use health::{check_vault_connectivity, check_vault_health, VaultHealthStatus};
