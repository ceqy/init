//! Error types for Vault adapter

use errors::AppError;

/// Convert vaultrs error to AppError
pub fn map_vault_error(err: impl std::fmt::Display, context: &str) -> AppError {
    let err_str = err.to_string();
    
    if err_str.contains("404") || err_str.contains("not found") {
        AppError::not_found(format!("{}: {}", context, err_str))
    } else if err_str.contains("403") || err_str.contains("permission denied") {
        AppError::forbidden(format!("{}: {}", context, err_str))
    } else if err_str.contains("401") || err_str.contains("unauthorized") {
        AppError::unauthenticated(format!("{}: {}", context, err_str))
    } else {
        // All other errors (connection, timeout, etc.) are external service errors
        AppError::external_service(format!("{}: {}", context, err_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_404_error() {
        let err = map_vault_error("404 not found", "Reading secret");
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_map_403_error() {
        let err = map_vault_error("403 permission denied", "Writing secret");
        assert!(matches!(err, AppError::Forbidden(_)));
    }

    #[test]
    fn test_map_401_error() {
        let err = map_vault_error("401 unauthorized", "Authentication");
        assert!(matches!(err, AppError::Unauthenticated(_)));
    }

    #[test]
    fn test_map_connection_error() {
        let err = map_vault_error("connection timeout", "Connecting to Vault");
        assert!(matches!(err, AppError::ExternalService(_)));
    }
}
