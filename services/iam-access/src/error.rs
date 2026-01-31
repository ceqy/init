use errors::AppError;
use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum AccessError {
    #[error("Role not found")]
    RoleNotFound,
    #[error("Permission not found")]
    PermissionNotFound,
    #[error("Policy not found")]
    PolicyNotFound,
    #[error("Role already exists")]
    RoleAlreadyExists,
    #[error("Permission already exists")]
    PermissionAlreadyExists,
    #[error("Policy already exists")]
    PolicyAlreadyExists,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl From<AccessError> for Status {
    fn from(error: AccessError) -> Self {
        match error {
            AccessError::RoleNotFound
            | AccessError::PermissionNotFound
            | AccessError::PolicyNotFound => Status::not_found(error.to_string()),
            AccessError::RoleAlreadyExists
            | AccessError::PermissionAlreadyExists
            | AccessError::PolicyAlreadyExists => Status::already_exists(error.to_string()),
            AccessError::DatabaseError(_) => Status::internal("Database error"),
            AccessError::Internal(msg) => Status::internal(msg),
        }
    }
}

impl From<AccessError> for AppError {
    fn from(error: AccessError) -> Self {
        match error {
            AccessError::RoleNotFound => AppError::NotFound("Role not found".to_string()),
            AccessError::PermissionNotFound => {
                AppError::NotFound("Permission not found".to_string())
            }
            AccessError::PolicyNotFound => AppError::NotFound("Policy not found".to_string()),
            AccessError::RoleAlreadyExists => AppError::Conflict("Role already exists".to_string()),
            AccessError::PermissionAlreadyExists => {
                AppError::Conflict("Permission already exists".to_string())
            }
            AccessError::PolicyAlreadyExists => {
                AppError::Conflict("Policy already exists".to_string())
            }
            AccessError::DatabaseError(e) => AppError::Internal(e.to_string()),
            AccessError::Internal(msg) => AppError::Internal(msg),
        }
    }
}
