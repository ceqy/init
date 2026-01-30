//! Service error types

use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

impl From<ServiceError> for Status {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound(msg) => Status::not_found(msg),
            ServiceError::InvalidInput(msg) => Status::invalid_argument(msg),
            ServiceError::Database(e) => Status::internal(e.to_string()),
            ServiceError::Internal(msg) => Status::internal(msg),
            ServiceError::Unauthorized(msg) => Status::unauthenticated(msg),
            ServiceError::Conflict(msg) => Status::already_exists(msg),
        }
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;
