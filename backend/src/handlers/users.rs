//! User handlers

use actix_web::{web, HttpRequest, HttpResponse};
use shared::dto::{ChangePasswordRequest, UpdateUserRequest};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::error::{ApiError, ApiResult};
use crate::middleware::get_user_id_from_request;
use crate::services::Services;

pub async fn get_current_user(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let user = services.users.get_user(&user_id).await?;
    Ok(HttpResponse::Ok().json(user))
}

pub async fn update_current_user(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    body: web::Json<UpdateUserRequest>,
) -> ApiResult<HttpResponse> {
    body.validate().map_err(ApiError::from)?;

    let user_id = get_user_id_from_request(&req, &services)?;
    let user = services.users.update_user(&user_id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(user))
}

pub async fn change_password(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    body: web::Json<ChangePasswordRequest>,
) -> ApiResult<HttpResponse> {
    body.validate().map_err(ApiError::from)?;

    let user_id = get_user_id_from_request(&req, &services)?;
    services
        .users
        .change_password(&user_id, &body.current_password, &body.new_password)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password changed successfully"
    })))
}

pub async fn get_user(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    // Verify requester is authenticated
    let _ = get_user_id_from_request(&req, &services)?;

    let user_id = path.into_inner();
    let user = services.users.get_user(&user_id).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    q: String,
    limit: Option<i64>,
    offset: Option<i64>,
}

pub async fn search_users(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    query: web::Query<SearchQuery>,
) -> ApiResult<HttpResponse> {
    // Verify requester is authenticated
    let _ = get_user_id_from_request(&req, &services)?;

    let users = services
        .users
        .search_users(&query.q, query.limit.unwrap_or(20), query.offset.unwrap_or(0))
        .await?;

    Ok(HttpResponse::Ok().json(users))
}
