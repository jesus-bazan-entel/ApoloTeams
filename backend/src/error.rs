//! Error handling for the backend

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use shared::error::{AppError, ErrorResponse};
use std::fmt;

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        ApiError(err)
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError(AppError::NotFoundError("Resource not found".to_string())),
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    ApiError(AppError::ConflictError("Resource already exists".to_string()))
                } else {
                    ApiError(AppError::DatabaseError(db_err.to_string()))
                }
            }
            _ => ApiError(AppError::DatabaseError(err.to_string())),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        ApiError(AppError::AuthenticationError(err.to_string()))
    }
}

impl From<argon2::password_hash::Error> for ApiError {
    fn from(err: argon2::password_hash::Error) -> Self {
        ApiError(AppError::AuthenticationError(err.to_string()))
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError(AppError::InternalError(err.to_string()))
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError(AppError::from(err))
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match &self.0 {
            AppError::AuthenticationError(_) => StatusCode::UNAUTHORIZED,
            AppError::AuthorizationError(_) => StatusCode::FORBIDDEN,
            AppError::NotFoundError(_) => StatusCode::NOT_FOUND,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::ConflictError(_) => StatusCode::CONFLICT,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AppError::FileUploadError(_) => StatusCode::BAD_REQUEST,
            AppError::WebSocketError(_) => StatusCode::BAD_REQUEST,
            AppError::DatabaseError(_) | AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let error_response: ErrorResponse = self.0.to_error_response();
        HttpResponse::build(self.status_code()).json(error_response)
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
