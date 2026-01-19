//! Search handlers

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use shared::dto::SearchResponse;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::get_user_id_from_request;
use crate::services::Services;

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    q: String,
    channel_id: Option<Uuid>,
    from_user_id: Option<Uuid>,
    from_date: Option<DateTime<Utc>>,
    to_date: Option<DateTime<Utc>>,
    limit: Option<i64>,
    offset: Option<i64>,
}

pub async fn search_messages(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    query: web::Query<SearchQuery>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;

    let (messages, total_count) = services
        .messages
        .search_messages(
            &user_id,
            &query.q,
            query.channel_id.as_ref(),
            query.from_user_id.as_ref(),
            query.from_date,
            query.to_date,
            query.limit.unwrap_or(20),
            query.offset.unwrap_or(0),
        )
        .await?;

    Ok(HttpResponse::Ok().json(SearchResponse {
        messages,
        total_count,
    }))
}
