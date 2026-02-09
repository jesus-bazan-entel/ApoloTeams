//! Call handlers

use actix_web::{web, HttpRequest, HttpResponse};
use shared::dto::{StartCallRequest, UpdateCallParticipantRequest, WebSocketMessage};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::error::ApiResult;
use crate::middleware::get_user_id_from_request;
use crate::services::Services;
use crate::websocket::WebSocketServer;

pub async fn start_call(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    ws_server: web::Data<Arc<WebSocketServer>>,
    body: web::Json<StartCallRequest>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;

    let (call, is_direct_call, target_user) = match (&body.channel_id, &body.target_user_id) {
        // Direct call to a user
        (None, Some(target_user_id)) => {
            let call = services
                .calls
                .start_direct_call(&user_id, target_user_id, body.call_type.clone())
                .await?;
            (call, true, Some(*target_user_id))
        }
        // Channel-based call
        (Some(channel_id), None) => {
            let call = services
                .calls
                .start_call(channel_id, &user_id, body.call_type.clone())
                .await?;
            (call, false, None)
        }
        // Both provided - use channel_id
        (Some(channel_id), Some(_)) => {
            let call = services
                .calls
                .start_call(channel_id, &user_id, body.call_type.clone())
                .await?;
            (call, false, None)
        }
        // Neither provided - error
        (None, None) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Either channel_id or target_user_id must be provided"
            })));
        }
    };

    // Notify about the new call via WebSocket
    let call_started_msg = WebSocketMessage::CallStarted { call: call.clone() };

    if is_direct_call {
        // For direct calls, send directly to the target user
        if let Some(target_id) = target_user {
            info!("start_call: sending CallStarted to target user {} (direct call {})", target_id, call.id);
            ws_server.send_to_user(&target_id, &call_started_msg);
        }
    } else {
        // For channel calls, send to ALL channel members from DB (not just WS subscribers)
        // because the callee may not have the channel open in their WebSocket
        match services.channels.get_member_user_ids(&call.channel_id).await {
            Ok(member_ids) => {
                info!("start_call: sending CallStarted to {} channel members (call {})", member_ids.len(), call.id);
                for member_id in &member_ids {
                    if *member_id != user_id {
                        ws_server.send_to_user(member_id, &call_started_msg);
                    }
                }
            }
            Err(e) => {
                warn!("start_call: failed to get channel members, falling back to broadcast: {}", e);
                ws_server.broadcast_to_channel(&call.channel_id, &call_started_msg, Some(&user_id));
            }
        }
    }

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
    ws_server: web::Data<Arc<WebSocketServer>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let call_id = path.into_inner();

    let call = services.calls.join_call(&call_id, &user_id).await?;

    // Find the participant that just joined
    if let Some(participant) = call.participants.iter().find(|p| p.user.id == user_id) {
        let msg = WebSocketMessage::ParticipantJoined {
            call_id,
            participant: participant.clone(),
        };
        info!("join_call: sending ParticipantJoined for user {} to call participants (call {})", user_id, call_id);
        ws_server.send_to_call_participants(&call_id, &msg, Some(&user_id));
    } else {
        warn!("join_call: user {} not found in participants of call {}", user_id, call_id);
    }

    Ok(HttpResponse::Ok().json(call))
}

pub async fn leave_call(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    ws_server: web::Data<Arc<WebSocketServer>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let call_id = path.into_inner();

    services.calls.leave_call(&call_id, &user_id).await?;

    // Notify other call participants
    let msg = WebSocketMessage::ParticipantLeft {
        call_id,
        user_id,
    };
    ws_server.send_to_call_participants(&call_id, &msg, Some(&user_id));

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Left call successfully"
    })))
}

pub async fn end_call(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    ws_server: web::Data<Arc<WebSocketServer>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let call_id = path.into_inner();

    // Get call info before ending to know the channel_id
    let call = services.calls.get_call(&call_id, &user_id).await?;
    let channel_id = call.channel_id;

    services.calls.end_call(&call_id, &user_id).await?;

    // Notify all call participants that the call has ended
    let msg = WebSocketMessage::CallEnded { call_id };
    ws_server.send_to_call_participants(&call_id, &msg, None);
    // Also send to all channel members (callee may not have joined the call subscription)
    ws_server.broadcast_to_channel(&channel_id, &msg, None);
    // Clean up call subscription
    ws_server.remove_call(&call_id);

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

pub async fn get_ice_servers(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    config: web::Data<AppConfig>,
) -> ApiResult<HttpResponse> {
    // Require authentication
    let _user_id = get_user_id_from_request(&req, &services)?;

    let mut ice_servers = vec![
        serde_json::json!({ "urls": "stun:stun.l.google.com:19302" }),
        serde_json::json!({ "urls": "stun:stun1.l.google.com:19302" }),
    ];

    if let Some(turn) = &config.turn {
        ice_servers.push(serde_json::json!({
            "urls": &turn.server_url,
            "username": &turn.username,
            "credential": &turn.credential,
        }));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "ice_servers": ice_servers
    })))
}
