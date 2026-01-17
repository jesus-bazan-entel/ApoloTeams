//! Notification handlers

use actix_web::{web, HttpRequest, HttpResponse};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::get_user_id_from_request;
use crate::services::Services;

#[derive(serde::Deserialize)]
pub struct ListNotificationsQuery {
    unread_only: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
}

pub async fn list_notifications(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    query: web::Query<ListNotificationsQuery>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;

    let notifications = services
        .notifications
        .list_notifications(
            &user_id,
            query.unread_only.unwrap_or(false),
            query.limit.unwrap_or(50),
            query.offset.unwrap_or(0),
        )
        .await?;

    Ok(HttpResponse::Ok().json(notifications))
}

pub async fn mark_as_read(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let notification_id = path.into_inner();

    services
        .notifications
        .mark_as_read(&notification_id, &user_id)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Notification marked as read"
    })))
}

pub async fn mark_all_as_read(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;

    services.notifications.mark_all_as_read(&user_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "All notifications marked as read"
    })))
}
