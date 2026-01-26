//! 服务错误定义

use cuba_errors::AppError;
use crate::shared::domain::value_objects::UsernameError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Session not found")]
    SessionNotFound,

    #[error("2FA required")]
    TwoFactorRequired,

    #[error("Invalid 2FA code")]
    Invalid2FACode,

    #[error("Password too weak")]
    PasswordTooWeak,
}

impl From<UsernameError> for AppError {
    fn from(error: UsernameError) -> Self {
        AppError::validation(error.to_string())
    }
}


impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => AppError::unauthorized("Invalid credentials"),
            AuthError::UserNotFound => AppError::not_found("User not found"),
            AuthError::UserAlreadyExists => AppError::conflict("User already exists"),
            AuthError::InvalidToken => AppError::unauthorized("Invalid token"),
            AuthError::TokenExpired => AppError::unauthorized("Token expired"),
            AuthError::SessionNotFound => AppError::not_found("Session not found"),
            AuthError::TwoFactorRequired => AppError::unauthorized("2FA required"),
            AuthError::Invalid2FACode => AppError::unauthorized("Invalid 2FA code"),
            AuthError::PasswordTooWeak => AppError::validation("Password too weak"),
        }
    }
}
