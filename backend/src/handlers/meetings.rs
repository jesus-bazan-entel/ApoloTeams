//! Meeting handlers

use actix_web::{web, HttpRequest, HttpResponse};
use shared::dto::{
    CalendarQuery, CreateMeetingRequest, MeetingInviteRequest, MeetingResponseRequest,
    UpdateMeetingRequest, WebSocketMessage,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::get_user_id_from_request;
use crate::services::Services;
use crate::websocket::WebSocketServer;

/// Create a new meeting
pub async fn create_meeting(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    ws_server: web::Data<Arc<WebSocketServer>>,
    body: web::Json<CreateMeetingRequest>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;

    let meeting = services
        .meetings
        .create_meeting(&user_id, body.into_inner())
        .await?;

    // Notify all invited participants via WebSocket
    for participant in &meeting.participants {
        if participant.user.id != user_id {
            let msg = WebSocketMessage::MeetingInvite {
                meeting: meeting.clone(),
            };
            ws_server.send_to_user(&participant.user.id, &msg);
        }
    }

    Ok(HttpResponse::Created().json(meeting))
}

/// Get a meeting by ID
pub async fn get_meeting(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let meeting_id = path.into_inner();

    let meeting = services.meetings.get_meeting(&meeting_id, &user_id).await?;
    Ok(HttpResponse::Ok().json(meeting))
}

/// Get all meetings for the current user
pub async fn get_my_meetings(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;

    let meetings = services.meetings.get_user_meetings(&user_id).await?;
    Ok(HttpResponse::Ok().json(meetings))
}

/// Get meetings in a date range (calendar view)
pub async fn get_calendar(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    query: web::Query<CalendarQuery>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;

    let meetings = services
        .meetings
        .get_meetings_in_range(&user_id, query.start_date, query.end_date)
        .await?;

    Ok(HttpResponse::Ok().json(meetings))
}

/// Update a meeting
pub async fn update_meeting(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    ws_server: web::Data<Arc<WebSocketServer>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateMeetingRequest>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let meeting_id = path.into_inner();

    let meeting = services
        .meetings
        .update_meeting(&meeting_id, &user_id, body.into_inner())
        .await?;

    // Notify all participants about the update
    for participant in &meeting.participants {
        let msg = WebSocketMessage::MeetingUpdated {
            meeting: meeting.clone(),
        };
        ws_server.send_to_user(&participant.user.id, &msg);
    }

    Ok(HttpResponse::Ok().json(meeting))
}

/// Cancel a meeting
pub async fn cancel_meeting(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    ws_server: web::Data<Arc<WebSocketServer>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let meeting_id = path.into_inner();

    // Get meeting to notify participants before cancelling
    let meeting = services.meetings.get_meeting(&meeting_id, &user_id).await?;

    services.meetings.cancel_meeting(&meeting_id, &user_id).await?;

    // Notify all participants about the cancellation
    for participant in &meeting.participants {
        let msg = WebSocketMessage::MeetingCancelled { meeting_id };
        ws_server.send_to_user(&participant.user.id, &msg);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Meeting cancelled successfully"
    })))
}

/// Delete a meeting
pub async fn delete_meeting(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    ws_server: web::Data<Arc<WebSocketServer>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let meeting_id = path.into_inner();

    // Get meeting to notify participants before deleting
    let meeting = services.meetings.get_meeting(&meeting_id, &user_id).await?;

    services.meetings.delete_meeting(&meeting_id, &user_id).await?;

    // Notify all participants about the cancellation
    for participant in &meeting.participants {
        let msg = WebSocketMessage::MeetingCancelled { meeting_id };
        ws_server.send_to_user(&participant.user.id, &msg);
    }

    Ok(HttpResponse::NoContent().finish())
}

/// Invite participants to a meeting
pub async fn invite_participants(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    ws_server: web::Data<Arc<WebSocketServer>>,
    path: web::Path<Uuid>,
    body: web::Json<MeetingInviteRequest>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let meeting_id = path.into_inner();

    let meeting = services
        .meetings
        .invite_participants(&meeting_id, &user_id, body.user_ids.clone())
        .await?;

    // Notify newly invited participants
    for new_user_id in &body.user_ids {
        let msg = WebSocketMessage::MeetingInvite {
            meeting: meeting.clone(),
        };
        ws_server.send_to_user(new_user_id, &msg);
    }

    Ok(HttpResponse::Ok().json(meeting))
}

/// Respond to a meeting invitation
pub async fn respond_to_meeting(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    ws_server: web::Data<Arc<WebSocketServer>>,
    path: web::Path<Uuid>,
    body: web::Json<MeetingResponseRequest>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let meeting_id = path.into_inner();

    let meeting = services
        .meetings
        .respond_to_meeting(&meeting_id, &user_id, body.response.clone())
        .await?;

    // Notify organizer about the response
    let msg = WebSocketMessage::MeetingUpdated {
        meeting: meeting.clone(),
    };
    ws_server.send_to_user(&meeting.organizer.id, &msg);

    Ok(HttpResponse::Ok().json(meeting))
}

/// Remove a participant from a meeting
pub async fn remove_participant(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<(Uuid, Uuid)>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let (meeting_id, participant_id) = path.into_inner();

    services
        .meetings
        .remove_participant(&meeting_id, &user_id, &participant_id)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}
