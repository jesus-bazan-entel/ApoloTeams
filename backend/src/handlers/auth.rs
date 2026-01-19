//! Authentication handlers

use actix_web::{web, HttpResponse};
use shared::dto::{LoginRequest, RefreshTokenRequest, RegisterRequest};
use std::sync::Arc;
use validator::Validate;

use crate::error::{ApiError, ApiResult};
use crate::services::Services;

pub async fn register(
    services: web::Data<Arc<Services>>,
    body: web::Json<RegisterRequest>,
) -> ApiResult<HttpResponse> {
    body.validate().map_err(ApiError::from)?;

    let response = services.auth.register(body.into_inner()).await?;
    Ok(HttpResponse::Created().json(response))
}

pub async fn login(
    services: web::Data<Arc<Services>>,
    body: web::Json<LoginRequest>,
) -> ApiResult<HttpResponse> {
    body.validate().map_err(ApiError::from)?;

    let response = services.auth.login(body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn refresh_token(
    services: web::Data<Arc<Services>>,
    body: web::Json<RefreshTokenRequest>,
) -> ApiResult<HttpResponse> {
    let response = services.auth.refresh_token(&body.refresh_token).await?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn logout() -> ApiResult<HttpResponse> {
    // In a stateless JWT setup, logout is handled client-side
    // For a more secure setup, you could implement token blacklisting
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
