//! Message handlers

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use shared::dto::{AddReactionRequest, SendMessageRequest, UpdateMessageRequest};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::error::{ApiError, ApiResult};
use crate::middleware::get_user_id_from_request;
use crate::services::Services;

#[derive(serde::Deserialize)]
pub struct ListMessagesQuery {
    limit: Option<i64>,
    before: Option<DateTime<Utc>>,
    after: Option<DateTime<Utc>>,
}

pub async fn list_messages(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
    query: web::Query<ListMessagesQuery>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let channel_id = path.into_inner();

    let messages = services
        .messages
        .list_messages(
            &channel_id,
            &user_id,
            query.limit.unwrap_or(50),
            query.before,
            query.after,
        )
        .await?;

    Ok(HttpResponse::Ok().json(messages))
}

pub async fn send_message(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
    body: web::Json<SendMessageRequest>,
) -> ApiResult<HttpResponse> {
    body.validate().map_err(ApiError::from)?;

    let user_id = get_user_id_from_request(&req, &services)?;
    let channel_id = path.into_inner();

    let message = services
        .messages
        .send_message(&channel_id, &user_id, body.into_inner())
        .await?;

    Ok(HttpResponse::Created().json(message))
}

#[derive(serde::Deserialize)]
pub struct MessagePath {
    channel_id: Uuid,
    message_id: Uuid,
}

pub async fn update_message(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<MessagePath>,
    body: web::Json<UpdateMessageRequest>,
) -> ApiResult<HttpResponse> {
    body.validate().map_err(ApiError::from)?;

    let user_id = get_user_id_from_request(&req, &services)?;
    let params = path.into_inner();

    let message = services
        .messages
        .update_message(&params.message_id, &user_id, body.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(message))
}

pub async fn delete_message(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<MessagePath>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let params = path.into_inner();

    services
        .messages
        .delete_message(&params.message_id, &user_id)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn add_reaction(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<MessagePath>,
    body: web::Json<AddReactionRequest>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let params = path.into_inner();

    services
        .messages
        .add_reaction(&params.message_id, &user_id, &body.emoji)
        .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Reaction added"
    })))
}

#[derive(serde::Deserialize)]
pub struct ReactionPath {
    channel_id: Uuid,
    message_id: Uuid,
    emoji: String,
}

pub async fn remove_reaction(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<ReactionPath>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let params = path.into_inner();

    services
        .messages
        .remove_reaction(&params.message_id, &user_id, &params.emoji)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}
