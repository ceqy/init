# adapter-vault Quick Start Guide

## 5-Minute Setup

### 1. Add Dependency

In your service's `Cargo.toml`:

```toml
[dependencies]
adapter-vault = { path = "../../crates/adapters/vault" }
errors = { workspace = true }
```

### 2. Set Environment Variables

```bash
export VAULT_ADDR=http://10.0.0.10:10018
export VAULT_ROLE_ID=3f32c27e-0922-7564-2b18-19be3fe91215
export VAULT_SECRET_ID=ddb9882e-2afc-7851-aa9c-071788918f87
```

Or create a `.env` file:

```env
VAULT_ADDR=http://10.0.0.10:10018
VAULT_ROLE_ID=3f32c27e-0922-7564-2b18-19be3fe91215
VAULT_SECRET_ID=ddb9882e-2afc-7851-aa9c-071788918f87
```

### 3. Basic Usage

```rust
use adapter_vault::{VaultClient, VaultConfigBuilder};
use errors::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Create client
    let config = VaultConfigBuilder::new(std::env::var("VAULT_ADDR")?)
        .with_approle(
            std::env::var("VAULT_ROLE_ID")?,
            std::env::var("VAULT_SECRET_ID")?
        )
        .build();
    
    let vault = VaultClient::new(config).await?;
    
    // Read database password
    let db_password = vault
        .get_secret_field("database/postgresql", "password")
        .await?;
    
    println!("Database password: {}", db_password);
    
    Ok(())
}
```

## Common Use Cases

### Load Database Credentials

```rust
async fn get_database_url(vault: &VaultClient) -> AppResult<String> {
    let host = vault.get_secret_field("database/postgresql", "host").await?;
    let port = vault.get_secret_field("database/postgresql", "port").await?;
    let username = vault.get_secret_field("database/postgresql", "username").await?;
    let password = vault.get_secret_field("database/postgresql", "password").await?;
    let database = vault.get_secret_field("database/postgresql", "database").await?;
    
    Ok(format!(
        "postgresql://{}:{}@{}:{}/{}",
        username, password, host, port, database
    ))
}
```

### Load Redis Credentials

```rust
async fn get_redis_url(vault: &VaultClient) -> AppResult<String> {
    let host = vault.get_secret_field("database/redis", "host").await?;
    let port = vault.get_secret_field("database/redis", "port").await?;
    let password = vault.get_secret_field("database/redis", "password").await?;
    
    Ok(format!("redis://:{}@{}:{}", password, host, port))
}
```

### Load All Service Credentials

```rust
use std::collections::HashMap;

async fn load_all_credentials(vault: &VaultClient) -> AppResult<HashMap<String, String>> {
    let mut creds = HashMap::new();
    
    // PostgreSQL
    let pg_url = get_database_url(vault).await?;
    creds.insert("postgres_url".to_string(), pg_url);
    
    // Redis
    let redis_url = get_redis_url(vault).await?;
    creds.insert("redis_url".to_string(), redis_url);
    
    // Add more as needed...
    
    Ok(creds)
}
```

### Health Check in Service

```rust
use adapter_vault::check_vault_health;

async fn check_dependencies(vault: &VaultClient) -> AppResult<()> {
    let health = check_vault_health(vault).await;
    
    if !health.is_healthy() {
        return Err(errors::AppError::external_service(
            format!("Vault is unhealthy: {:?}", health.error)
        ));
    }
    
    Ok(())
}
```

## Integration with Bootstrap

In your service's `main.rs`:

```rust
use adapter_vault::{VaultClient, VaultConfigBuilder};
use cuba_bootstrap::{run, Infrastructure};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run("config", |infra: Infrastructure| async move {
        // Create Vault client
        let vault_config = VaultConfigBuilder::new(
            std::env::var("VAULT_ADDR")?
        )
        .with_approle(
            std::env::var("VAULT_ROLE_ID")?,
            std::env::var("VAULT_SECRET_ID")?
        )
        .build();
        
        let vault = VaultClient::new(vault_config).await?;
        
        // Load credentials from Vault
        let db_password = vault
            .get_secret_field("database/postgresql", "password")
            .await?;
        
        // Use credentials to create connections
        // ... rest of your service setup
        
        YourServiceServer::new(service)
    })
    .await
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires Vault
    async fn test_load_credentials() {
        let config = VaultConfigBuilder::new("http://10.0.0.10:10018")
            .with_approle(
                std::env::var("VAULT_ROLE_ID").unwrap(),
                std::env::var("VAULT_SECRET_ID").unwrap()
            )
            .build();
        
        let vault = VaultClient::new(config).await.unwrap();
        let password = vault
            .get_secret_field("database/postgresql", "password")
            .await
            .unwrap();
        
        assert!(!password.is_empty());
    }
}
```

### Run Tests

```bash
# Unit tests (no Vault required)
cargo test -p adapter-vault --lib

# Integration tests (requires Vault)
export VAULT_ADDR=http://10.0.0.10:10018
export VAULT_ROLE_ID=your-role-id
export VAULT_SECRET_ID=your-secret-id
cargo test -p adapter-vault --test integration_test -- --ignored
```

## Troubleshooting

### Error: "Failed to create Vault client"
- Check `VAULT_ADDR` is correct
- Verify Vault is running: `curl $VAULT_ADDR/v1/sys/health`

### Error: "AppRole authentication failed"
- Verify `VAULT_ROLE_ID` and `VAULT_SECRET_ID` are correct
- Check AppRole is enabled: `vault auth list`

### Error: "Secret not found"
- Verify secret path is correct
- Check you have read permissions: `vault kv get secret/path/to/secret`

### Error: "Permission denied"
- Check AppRole policy grants required permissions
- View policy: `vault policy read your-policy-name`

## Best Practices

1. **Always use environment variables** for Vault credentials
2. **Never log secrets** - use masking in logs
3. **Implement health checks** in your services
4. **Handle errors gracefully** - provide fallbacks if needed
5. **Use specific field reads** when you only need one value
6. **Cache credentials** if appropriate (with TTL)
7. **Rotate secrets regularly** using Vault's rotation features

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Check [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) for architecture details
- Run the example: `cargo run --example basic_usage`
- Integrate with your service's bootstrap code

## Support

For issues or questions:
1. Check the [README.md](README.md) documentation
2. Review the [example code](examples/basic_usage.rs)
3. Run integration tests to verify setup
4. Check Vault server logs for authentication issues
