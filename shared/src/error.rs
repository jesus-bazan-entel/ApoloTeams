//! Error types for Rust Teams application
//! 
//! Defines all error types used across the application.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Application error types
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Authorization failed: {0}")]
    AuthorizationError(String),

    #[error("Resource not found: {0}")]
    NotFoundError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Conflict error: {0}")]
    ConflictError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("File upload error: {0}")]
    FileUploadError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}

/// Error response structure for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::AuthenticationError(_) => "AUTHENTICATION_ERROR",
            AppError::AuthorizationError(_) => "AUTHORIZATION_ERROR",
            AppError::NotFoundError(_) => "NOT_FOUND",
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::ConflictError(_) => "CONFLICT",
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::InternalError(_) => "INTERNAL_ERROR",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            AppError::FileUploadError(_) => "FILE_UPLOAD_ERROR",
            AppError::WebSocketError(_) => "WEBSOCKET_ERROR",
        }
    }

    pub fn to_error_response(&self) -> ErrorResponse {
        ErrorResponse {
            error: ErrorDetail {
                code: self.error_code().to_string(),
                message: self.to_string(),
                details: None,
            },
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| {
                    format!(
                        "{}: {}",
                        field,
                        e.message.as_ref().map(|m| m.to_string()).unwrap_or_default()
                    )
                })
            })
            .collect();
        AppError::ValidationError(messages.join(", "))
    }
}
