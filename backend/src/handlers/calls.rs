//! Call handlers

use actix_web::{web, HttpRequest, HttpResponse};
use shared::dto::{StartCallRequest, UpdateCallParticipantRequest};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::get_user_id_from_request;
use crate::services::Services;

pub async fn start_call(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    body: web::Json<StartCallRequest>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;

    let call = services
        .calls
        .start_call(&body.channel_id, &user_id, body.call_type)
        .await?;

    Ok(HttpResponse::Created().json(call))
}

pub async fn get_call(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let call_id = path.into_inner();

    let call = services.calls.get_call(&call_id, &user_id).await?;
    Ok(HttpResponse::Ok().json(call))
}

pub async fn join_call(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let call_id = path.into_inner();

    let call = services.calls.join_call(&call_id, &user_id).await?;
    Ok(HttpResponse::Ok().json(call))
}

pub async fn leave_call(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let call_id = path.into_inner();

    services.calls.leave_call(&call_id, &user_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Left call successfully"
    })))
}

pub async fn end_call(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let call_id = path.into_inner();

    services.calls.end_call(&call_id, &user_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Call ended successfully"
    })))
}

pub async fn update_participant(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateCallParticipantRequest>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let call_id = path.into_inner();

    let participant = services
        .calls
        .update_participant(&call_id, &user_id, body.is_muted, body.is_video_enabled)
        .await?;

    Ok(HttpResponse::Ok().json(participant))
}
