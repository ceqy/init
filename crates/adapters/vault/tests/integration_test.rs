//! Integration tests for Vault adapter
//!
//! These tests require a running Vault server with the following setup:
//! - Vault running at http://10.0.0.10:10018
//! - AppRole authentication enabled
//! - Environment variables: VAULT_ROLE_ID, VAULT_SECRET_ID

use adapter_vault::{VaultClient, VaultConfigBuilder};
use std::collections::HashMap;

#[tokio::test]
#[ignore] // Requires running Vault server
async fn test_full_secret_lifecycle() {
    // Setup
    let config = VaultConfigBuilder::new("http://10.0.0.10:10018")
        .with_approle(
            std::env::var("VAULT_ROLE_ID").expect("VAULT_ROLE_ID not set"),
            std::env::var("VAULT_SECRET_ID").expect("VAULT_SECRET_ID not set"),
        )
        .build();

    let client = VaultClient::new(config)
        .await
        .expect("Failed to create Vault client");

    // Test 1: Write secret
    let mut data = HashMap::new();
    data.insert("username".to_string(), "integration_test_user".to_string());
    data.insert("password".to_string(), "super_secret_password".to_string());
    data.insert("api_key".to_string(), "test_api_key_12345".to_string());

    client
        .set_secret("test/integration/credentials", data.clone())
        .await
        .expect("Failed to write secret");

    // Test 2: Read full secret
    let retrieved = client
        .get_secret("test/integration/credentials")
        .await
        .expect("Failed to read secret");

    assert_eq!(retrieved.get("username").unwrap(), "integration_test_user");
    assert_eq!(retrieved.get("password").unwrap(), "super_secret_password");
    assert_eq!(retrieved.get("api_key").unwrap(), "test_api_key_12345");

    // Test 3: Read specific field
    let username = client
        .get_secret_field("test/integration/credentials", "username")
        .await
        .expect("Failed to read username field");

    assert_eq!(username, "integration_test_user");

    // Test 4: Update secret
    let mut updated_data = HashMap::new();
    updated_data.insert("username".to_string(), "updated_user".to_string());
    updated_data.insert("password".to_string(), "new_password".to_string());

    client
        .set_secret("test/integration/credentials", updated_data)
        .await
        .expect("Failed to update secret");

    let updated = client
        .get_secret("test/integration/credentials")
        .await
        .expect("Failed to read updated secret");

    assert_eq!(updated.get("username").unwrap(), "updated_user");
    assert_eq!(updated.get("password").unwrap(), "new_password");

    // Test 5: Delete secret
    client
        .delete_secret("test/integration/credentials")
        .await
        .expect("Failed to delete secret");

    // Test 6: Verify deletion
    let result = client.get_secret("test/integration/credentials").await;
    assert!(result.is_err(), "Secret should not exist after deletion");
}

#[tokio::test]
#[ignore] // Requires running Vault server
async fn test_health_check() {
    let config = VaultConfigBuilder::new("http://10.0.0.10:10018")
        .with_approle(
            std::env::var("VAULT_ROLE_ID").expect("VAULT_ROLE_ID not set"),
            std::env::var("VAULT_SECRET_ID").expect("VAULT_SECRET_ID not set"),
        )
        .build();

    let client = VaultClient::new(config)
        .await
        .expect("Failed to create Vault client");

    let health = client.health_check().await;
    assert!(health.is_ok(), "Health check should pass");
    assert!(health.unwrap(), "Vault should be accessible");
}

#[tokio::test]
#[ignore] // Requires running Vault server
async fn test_nonexistent_secret() {
    let config = VaultConfigBuilder::new("http://10.0.0.10:10018")
        .with_approle(
            std::env::var("VAULT_ROLE_ID").expect("VAULT_ROLE_ID not set"),
            std::env::var("VAULT_SECRET_ID").expect("VAULT_SECRET_ID not set"),
        )
        .build();

    let client = VaultClient::new(config)
        .await
        .expect("Failed to create Vault client");

    let result = client.get_secret("test/nonexistent/path").await;
    assert!(result.is_err(), "Should return error for nonexistent secret");
}

#[tokio::test]
#[ignore] // Requires running Vault server
async fn test_missing_field() {
    let config = VaultConfigBuilder::new("http://10.0.0.10:10018")
        .with_approle(
            std::env::var("VAULT_ROLE_ID").expect("VAULT_ROLE_ID not set"),
            std::env::var("VAULT_SECRET_ID").expect("VAULT_SECRET_ID not set"),
        )
        .build();

    let client = VaultClient::new(config)
        .await
        .expect("Failed to create Vault client");

    // Create a secret with limited fields
    let mut data = HashMap::new();
    data.insert("field1".to_string(), "value1".to_string());

    client
        .set_secret("test/integration/limited", data)
        .await
        .expect("Failed to write secret");

    // Try to read a field that doesn't exist
    let result = client
        .get_secret_field("test/integration/limited", "nonexistent_field")
        .await;

    assert!(result.is_err(), "Should return error for missing field");

    // Cleanup
    client
        .delete_secret("test/integration/limited")
        .await
        .expect("Failed to cleanup");
}

#[tokio::test]
#[ignore] // Requires running Vault server
async fn test_database_credentials() {
    let config = VaultConfigBuilder::new("http://10.0.0.10:10018")
        .with_approle(
            std::env::var("VAULT_ROLE_ID").expect("VAULT_ROLE_ID not set"),
            std::env::var("VAULT_SECRET_ID").expect("VAULT_SECRET_ID not set"),
        )
        .build();

    let client = VaultClient::new(config)
        .await
        .expect("Failed to create Vault client");

    // Read PostgreSQL credentials (should already exist from setup)
    let pg_creds = client
        .get_secret("database/postgresql")
        .await
        .expect("Failed to read PostgreSQL credentials");

    assert!(pg_creds.contains_key("host"), "Should have host field");
    assert!(pg_creds.contains_key("port"), "Should have port field");
    assert!(pg_creds.contains_key("username"), "Should have username field");
    assert!(pg_creds.contains_key("password"), "Should have password field");

    // Read specific fields
    let host = client
        .get_secret_field("database/postgresql", "host")
        .await
        .expect("Failed to read host");

    assert_eq!(host, "10.0.0.10");
}
