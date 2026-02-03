//! Basic usage example for adapter-vault
//!
//! Run with:
//! ```bash
//! export VAULT_ADDR=http://10.0.0.10:10018
//! export VAULT_ROLE_ID=your-role-id
//! export VAULT_SECRET_ID=your-secret-id
//! cargo run --example basic_usage
//! ```

use adapter_vault::{check_vault_health, VaultClient, VaultConfigBuilder};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Vault Adapter Basic Usage Example ===\n");

    // 1. Create configuration from environment variables
    println!("1. Creating Vault client configuration...");
    let config = VaultConfigBuilder::new(
        std::env::var("VAULT_ADDR").unwrap_or_else(|_| "http://10.0.0.10:10018".to_string()),
    )
    .with_approle(
        std::env::var("VAULT_ROLE_ID")?,
        std::env::var("VAULT_SECRET_ID")?,
    )
    .with_mount_path("secret")
    .build();

    println!("   Vault endpoint: {}", config.endpoint);
    println!("   Mount path: {}", config.mount_path);

    // 2. Create Vault client
    println!("\n2. Connecting to Vault...");
    let client = VaultClient::new(config).await?;
    println!("   ✓ Successfully connected and authenticated");

    // 3. Health check
    println!("\n3. Performing health check...");
    let health = check_vault_health(&client).await;
    if health.is_healthy() {
        println!("   ✓ Vault is healthy");
        if let Some(response_time) = health.response_time_ms {
            println!("   Response time: {}ms", response_time);
        }
    } else {
        println!("   ✗ Vault is unhealthy: {:?}", health.error);
        return Ok(());
    }

    // 4. Write a secret
    println!("\n4. Writing a test secret...");
    let mut data = HashMap::new();
    data.insert("username".to_string(), "demo_user".to_string());
    data.insert("password".to_string(), "demo_password_123".to_string());
    data.insert("api_key".to_string(), "demo_api_key_xyz".to_string());

    client.set_secret("demo/example/credentials", data).await?;
    println!("   ✓ Secret written to: demo/example/credentials");

    // 5. Read the secret
    println!("\n5. Reading the secret...");
    let secret = client.get_secret("demo/example/credentials").await?;
    println!("   ✓ Secret retrieved:");
    for (key, value) in &secret {
        // Mask sensitive values
        let masked_value = if key == "password" || key == "api_key" {
            "********"
        } else {
            value
        };
        println!("     {}: {}", key, masked_value);
    }

    // 6. Read specific field
    println!("\n6. Reading specific field (username)...");
    let username = client
        .get_secret_field("demo/example/credentials", "username")
        .await?;
    println!("   ✓ Username: {}", username);

    // 7. Read database credentials (if they exist)
    println!("\n7. Reading database credentials...");
    match client.get_secret("database/postgresql").await {
        Ok(db_creds) => {
            println!("   ✓ PostgreSQL credentials found:");
            println!("     Host: {}", db_creds.get("host").unwrap_or(&"N/A".to_string()));
            println!("     Port: {}", db_creds.get("port").unwrap_or(&"N/A".to_string()));
            println!("     Username: {}", db_creds.get("username").unwrap_or(&"N/A".to_string()));
            println!("     Password: ********");
        }
        Err(e) => {
            println!("   ℹ Database credentials not found: {}", e);
        }
    }

    // 8. Delete the test secret
    println!("\n8. Cleaning up test secret...");
    client.delete_secret("demo/example/credentials").await?;
    println!("   ✓ Secret deleted");

    // 9. Verify deletion
    println!("\n9. Verifying deletion...");
    match client.get_secret("demo/example/credentials").await {
        Ok(_) => println!("   ✗ Secret still exists (unexpected)"),
        Err(_) => println!("   ✓ Secret successfully deleted"),
    }

    println!("\n=== Example completed successfully ===");

    Ok(())
}
