//! cuba-errors - 统一错误处理
//!
//! 基于 RFC 7807 Problem Details 规范

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 应用错误类型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("External service error: {0}")]
    ExternalService(String),
    
    #[error("Failed precondition: {0}")]
    FailedPrecondition(String),
    
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
}

impl AppError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }

    pub fn external_service(msg: impl Into<String>) -> Self {
        Self::ExternalService(msg.into())
    }
    
    pub fn unauthenticated(msg: impl Into<String>) -> Self {
        Self::Unauthenticated(msg.into())
    }
    
    pub fn failed_precondition(msg: impl Into<String>) -> Self {
        Self::FailedPrecondition(msg.into())
    }
    
    pub fn resource_exhausted(msg: impl Into<String>) -> Self {
        Self::ResourceExhausted(msg.into())
    }

    /// 转换为 HTTP 状态码
    pub fn status_code(&self) -> u16 {
        match self {
            Self::NotFound(_) => 404,
            Self::Validation(_) => 400,
            Self::Unauthorized(_) => 401,
            Self::Forbidden(_) => 403,
            Self::Conflict(_) => 409,
            Self::Internal(_) => 500,
            Self::Database(_) => 500,
            Self::ExternalService(_) => 502,
            Self::Unauthenticated(_) => 401,
            Self::FailedPrecondition(_) => 412,
            Self::ResourceExhausted(_) => 429,
        }
    }

    /// 转换为 gRPC 状态码
    pub fn grpc_code(&self) -> tonic::Code {
        match self {
            Self::NotFound(_) => tonic::Code::NotFound,
            Self::Validation(_) => tonic::Code::InvalidArgument,
            Self::Unauthorized(_) => tonic::Code::Unauthenticated,
            Self::Forbidden(_) => tonic::Code::PermissionDenied,
            Self::Conflict(_) => tonic::Code::AlreadyExists,
            Self::Internal(_) => tonic::Code::Internal,
            Self::Database(_) => tonic::Code::Internal,
            Self::ExternalService(_) => tonic::Code::Unavailable,
            Self::Unauthenticated(_) => tonic::Code::Unauthenticated,
            Self::FailedPrecondition(_) => tonic::Code::FailedPrecondition,
            Self::ResourceExhausted(_) => tonic::Code::ResourceExhausted,
        }
    }

    /// 转换为 Problem Details
    pub fn to_problem_details(&self) -> ProblemDetails {
        ProblemDetails {
            r#type: self.problem_type(),
            title: self.problem_title(),
            status: self.status_code(),
            detail: self.to_string(),
            instance: None,
        }
    }

    fn problem_type(&self) -> String {
        match self {
            Self::NotFound(_) => "https://api.cuba-erp.com/problems/not-found".to_string(),
            Self::Validation(_) => "https://api.cuba-erp.com/problems/validation".to_string(),
            Self::Unauthorized(_) => "https://api.cuba-erp.com/problems/unauthorized".to_string(),
            Self::Forbidden(_) => "https://api.cuba-erp.com/problems/forbidden".to_string(),
            Self::Conflict(_) => "https://api.cuba-erp.com/problems/conflict".to_string(),
            Self::Internal(_) => "https://api.cuba-erp.com/problems/internal".to_string(),
            Self::Database(_) => "https://api.cuba-erp.com/problems/database".to_string(),
            Self::ExternalService(_) => {
                "https://api.cuba-erp.com/problems/external-service".to_string()
            }
            Self::Unauthenticated(_) => "https://api.cuba-erp.com/problems/unauthenticated".to_string(),
            Self::FailedPrecondition(_) => "https://api.cuba-erp.com/problems/failed-precondition".to_string(),
            Self::ResourceExhausted(_) => "https://api.cuba-erp.com/problems/resource-exhausted".to_string(),
        }
    }

    fn problem_title(&self) -> String {
        match self {
            Self::NotFound(_) => "Resource Not Found".to_string(),
            Self::Validation(_) => "Validation Error".to_string(),
            Self::Unauthorized(_) => "Unauthorized".to_string(),
            Self::Forbidden(_) => "Forbidden".to_string(),
            Self::Conflict(_) => "Conflict".to_string(),
            Self::Internal(_) => "Internal Server Error".to_string(),
            Self::Database(_) => "Database Error".to_string(),
            Self::ExternalService(_) => "External Service Error".to_string(),
            Self::Unauthenticated(_) => "Unauthenticated".to_string(),
            Self::FailedPrecondition(_) => "Failed Precondition".to_string(),
            Self::ResourceExhausted(_) => "Resource Exhausted".to_string(),
        }
    }
}

impl From<AppError> for tonic::Status {
    fn from(err: AppError) -> Self {
        tonic::Status::new(err.grpc_code(), err.to_string())
    }
}

/// RFC 7807 Problem Details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    pub r#type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

/// Result 类型别名
pub type AppResult<T> = Result<T, AppError>;
