# adapter-vault

HashiCorp Vault adapter for unified secret management in the ERP system.

## Features

- **AppRole Authentication**: Secure authentication using AppRole method
- **KV v2 Secrets Engine**: Support for versioned key-value secrets
- **Health Checking**: Built-in health check functionality
- **Error Mapping**: Automatic conversion to `AppError` for consistent error handling
- **Async/Await**: Fully asynchronous API using Tokio
- **Type Safety**: Strong typing with Rust's type system

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
adapter-vault = { path = "../../crates/adapters/vault" }
```

## Usage

### Basic Setup

```rust
use adapter_vault::{VaultClient, VaultConfigBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = VaultConfigBuilder::new("http://10.0.0.10:10018")
        .with_approle("your-role-id", "your-secret-id")
        .with_mount_path("secret")
        .build();

    // Create client
    let client = VaultClient::new(config).await?;

    Ok(())
}
```

### Reading Secrets

```rust
use std::collections::HashMap;

// Read entire secret
let secret: HashMap<String, String> = client
    .get_secret("database/postgresql")
    .await?;

println!("Host: {}", secret.get("host").unwrap());
println!("Port: {}", secret.get("port").unwrap());

// Read specific field
let password = client
    .get_secret_field("database/postgresql", "password")
    .await?;

println!("Password: {}", password);
```

### Writing Secrets

```rust
use std::collections::HashMap;

let mut data = HashMap::new();
data.insert("username".to_string(), "admin".to_string());
data.insert("password".to_string(), "secret123".to_string());

client.set_secret("app/credentials", data).await?;
```

### Deleting Secrets

```rust
client.delete_secret("app/credentials").await?;
```

### Health Check

```rust
use adapter_vault::check_vault_health;

let health_status = check_vault_health(&client).await;

if health_status.is_healthy() {
    println!("Vault is healthy");
} else {
    println!("Vault is unhealthy: {:?}", health_status.error);
}
```

## Configuration

### VaultConfig Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `endpoint` | String | `http://localhost:8200` | Vault server URL |
| `role_id` | String | - | AppRole role ID |
| `secret_id` | String | - | AppRole secret ID |
| `mount_path` | String | `secret` | KV secrets engine mount path |
| `connection_timeout_secs` | u64 | 10 | Connection timeout in seconds |
| `request_timeout_secs` | u64 | 30 | Request timeout in seconds |
| `max_retries` | u32 | 3 | Maximum retry attempts |

### Using Builder Pattern

```rust
let config = VaultConfigBuilder::new("http://vault:8200")
    .with_approle("role-id", "secret-id")
    .with_mount_path("kv")
    .with_connection_timeout(20)
    .with_request_timeout(60)
    .with_max_retries(5)
    .build();
```

### Using Environment Variables

```rust
use std::env;

let config = VaultConfigBuilder::new(env::var("VAULT_ADDR")?)
    .with_approle(
        env::var("VAULT_ROLE_ID")?,
        env::var("VAULT_SECRET_ID")?
    )
    .build();
```

## Error Handling

All errors are mapped to `AppError` from the `errors` crate:

```rust
use errors::AppError;

match client.get_secret("path/to/secret").await {
    Ok(secret) => println!("Got secret: {:?}", secret),
    Err(AppError::NotFound(msg)) => println!("Secret not found: {}", msg),
    Err(AppError::Forbidden(msg)) => println!("Permission denied: {}", msg),
    Err(AppError::Unauthenticated(msg)) => println!("Auth failed: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

## Testing

### Unit Tests

```bash
cargo test -p adapter-vault --lib
```

### Integration Tests

Integration tests require a running Vault server:

```bash
# Set environment variables
export VAULT_ADDR=http://10.0.0.10:10018
export VAULT_ROLE_ID=your-role-id
export VAULT_SECRET_ID=your-secret-id

# Run integration tests
cargo test -p adapter-vault --test integration_test -- --ignored
```

## Architecture

The adapter follows a modular structure:

```
adapter-vault/
├── src/
│   ├── lib.rs          # Public API exports
│   ├── config.rs       # Configuration and builder
│   ├── client.rs       # Vault client implementation
│   ├── error.rs        # Error mapping
│   └── health.rs       # Health check functionality
└── tests/
    └── integration_test.rs  # Integration tests
```

## Best Practices

1. **Use Builder Pattern**: Always use `VaultConfigBuilder` for configuration
2. **Handle Errors**: Always handle errors appropriately, don't unwrap
3. **Health Checks**: Implement health checks in your services
4. **Secret Rotation**: Regularly rotate secrets in Vault
5. **Least Privilege**: Use AppRole policies with minimal required permissions

## Example: Database Connection

```rust
use adapter_vault::{VaultClient, VaultConfigBuilder};
use sqlx::PgPool;

async fn create_db_pool(vault: &VaultClient) -> Result<PgPool, Box<dyn std::error::Error>> {
    // Read database credentials from Vault
    let host = vault.get_secret_field("database/postgresql", "host").await?;
    let port = vault.get_secret_field("database/postgresql", "port").await?;
    let username = vault.get_secret_field("database/postgresql", "username").await?;
    let password = vault.get_secret_field("database/postgresql", "password").await?;
    let database = vault.get_secret_field("database/postgresql", "database").await?;

    // Build connection string
    let connection_string = format!(
        "postgresql://{}:{}@{}:{}/{}",
        username, password, host, port, database
    );

    // Create pool
    let pool = PgPool::connect(&connection_string).await?;

    Ok(pool)
}
```

## Security Considerations

1. **Never log secrets**: Secrets should never appear in logs
2. **Secure transport**: Always use HTTPS in production
3. **Rotate credentials**: Regularly rotate AppRole credentials
4. **Audit access**: Enable Vault audit logging
5. **Least privilege**: Grant minimal required permissions

## License

See workspace LICENSE file.
