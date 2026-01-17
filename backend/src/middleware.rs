//! Authentication middleware

use actix_web::{dev::ServiceRequest, HttpMessage};
use shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

use crate::services::Services;

/// Extract user ID from Authorization header
pub async fn extract_user_id(
    req: &ServiceRequest,
    services: &Arc<Services>,
) -> Result<Uuid, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::AuthenticationError("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::AuthenticationError("Invalid authorization header format".to_string()))?;

    services.auth.verify_access_token(token)
}

/// Extension trait to get user ID from request
pub trait RequestExt {
    fn user_id(&self) -> Option<Uuid>;
}

impl RequestExt for actix_web::HttpRequest {
    fn user_id(&self) -> Option<Uuid> {
        self.extensions().get::<Uuid>().copied()
    }
}

/// Helper function to extract user ID from request headers
pub fn get_user_id_from_request(
    req: &actix_web::HttpRequest,
    services: &Arc<Services>,
) -> Result<Uuid, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::AuthenticationError("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::AuthenticationError("Invalid authorization header format".to_string()))?;

    services.auth.verify_access_token(token)
}
