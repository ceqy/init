use crate::DatabaseConfig;
use secrecy::Secret;

#[test]
fn test_secret_redaction() {
    let secret = Secret::new("my_secret_password".to_string());
    let debug_output = format!("{:?}", secret);
    assert!(debug_output.contains("Secret([REDACTED"));
    assert!(!debug_output.contains("my_secret_password"));
}

#[test]
fn test_config_struct_redaction() {
    let config = DatabaseConfig {
        url: Secret::new("postgres://user:pass@localhost:5432/db".to_string()),
        max_connections: 10,
        read_url: None,
        read_max_connections: 20,
    };
    let debug_output = format!("{:?}", config);
    assert!(!debug_output.contains("pass"));
    assert!(debug_output.contains("Secret([REDACTED"));
}
