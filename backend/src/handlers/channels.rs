//! Channel handlers

use actix_web::{web, HttpRequest, HttpResponse};
use shared::dto::{CreateChannelRequest, UpdateChannelRequest};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::error::{ApiError, ApiResult};
use crate::middleware::get_user_id_from_request;
use crate::services::Services;

pub async fn list_channels(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let channels = services.channels.list_user_channels(&user_id).await?;
    Ok(HttpResponse::Ok().json(channels))
}

pub async fn list_team_channels(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let team_id = path.into_inner();

    let channels = services.channels.list_team_channels(&team_id, &user_id).await?;
    Ok(HttpResponse::Ok().json(channels))
}

pub async fn create_channel(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    body: web::Json<CreateChannelRequest>,
) -> ApiResult<HttpResponse> {
    body.validate().map_err(ApiError::from)?;

    let user_id = get_user_id_from_request(&req, &services)?;
    let channel = services.channels.create_channel(&user_id, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(channel))
}

#[derive(serde::Deserialize)]
pub struct CreateDmRequest {
    user_id: Uuid,
}

pub async fn create_dm_channel(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    body: web::Json<CreateDmRequest>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let channel = services.channels.create_dm_channel(&user_id, &body.user_id).await?;
    Ok(HttpResponse::Created().json(channel))
}

pub async fn get_channel(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let channel_id = path.into_inner();

    let channel = services.channels.get_channel(&channel_id, &user_id).await?;
    Ok(HttpResponse::Ok().json(channel))
}

pub async fn update_channel(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateChannelRequest>,
) -> ApiResult<HttpResponse> {
    body.validate().map_err(ApiError::from)?;

    let user_id = get_user_id_from_request(&req, &services)?;
    let channel_id = path.into_inner();

    let channel = services
        .channels
        .update_channel(&channel_id, &user_id, body.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(channel))
}

pub async fn delete_channel(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let channel_id = path.into_inner();

    services.channels.delete_channel(&channel_id, &user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_channel_members(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let channel_id = path.into_inner();

    let members = services.channels.list_members(&channel_id, &user_id).await?;
    Ok(HttpResponse::Ok().json(members))
}

#[derive(serde::Deserialize)]
pub struct AddMemberRequest {
    user_id: Uuid,
}

pub async fn add_channel_member(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
    body: web::Json<AddMemberRequest>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let channel_id = path.into_inner();

    let member = services
        .channels
        .add_member(&channel_id, &user_id, &body.user_id)
        .await?;
    Ok(HttpResponse::Created().json(member))
}

#[derive(serde::Deserialize)]
pub struct ChannelMemberPath {
    channel_id: Uuid,
    user_id: Uuid,
}

pub async fn remove_channel_member(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<ChannelMemberPath>,
) -> ApiResult<HttpResponse> {
    let requester_id = get_user_id_from_request(&req, &services)?;
    let params = path.into_inner();

    services
        .channels
        .remove_member(&params.channel_id, &requester_id, &params.user_id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn mark_as_read(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let channel_id = path.into_inner();

    services.channels.mark_as_read(&channel_id, &user_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Channel marked as read"
    })))
}
