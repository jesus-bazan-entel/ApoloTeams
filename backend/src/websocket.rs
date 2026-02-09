//! WebSocket handler for real-time communication

use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;
use dashmap::DashMap;
use futures_util::StreamExt;
use shared::dto::WebSocketMessage;
use shared::models::UserStatus;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::Services;

/// WebSocket server managing all connections
pub struct WebSocketServer {
    /// Map of user_id to their sender channel
    connections: DashMap<Uuid, broadcast::Sender<String>>,
    /// Map of channel_id to set of user_ids subscribed to it
    channel_subscriptions: DashMap<Uuid, HashSet<Uuid>>,
    /// Map of user_id to their current status
    user_statuses: DashMap<Uuid, UserStatus>,
    /// Map of call_id to set of user_ids participating in the call
    call_subscriptions: DashMap<Uuid, HashSet<Uuid>>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
            channel_subscriptions: DashMap::new(),
            user_statuses: DashMap::new(),
            call_subscriptions: DashMap::new(),
        }
    }

    /// Register a new connection
    pub fn register(&self, user_id: Uuid) -> broadcast::Receiver<String> {
        let (tx, rx) = broadcast::channel(100);
        self.connections.insert(user_id, tx);
        self.user_statuses.insert(user_id, UserStatus::Online);
        rx
    }

    /// Unregister a connection
    pub fn unregister(&self, user_id: &Uuid) {
        self.connections.remove(user_id);
        self.user_statuses.insert(*user_id, UserStatus::Offline);

        // Remove from all channel subscriptions
        for mut entry in self.channel_subscriptions.iter_mut() {
            entry.value_mut().remove(user_id);
        }

        // Remove from all call subscriptions
        for mut entry in self.call_subscriptions.iter_mut() {
            entry.value_mut().remove(user_id);
        }
    }

    /// Subscribe user to a channel
    pub fn subscribe_to_channel(&self, channel_id: Uuid, user_id: Uuid) {
        self.channel_subscriptions
            .entry(channel_id)
            .or_insert_with(HashSet::new)
            .insert(user_id);
    }

    /// Unsubscribe user from a channel
    pub fn unsubscribe_from_channel(&self, channel_id: &Uuid, user_id: &Uuid) {
        if let Some(mut subscribers) = self.channel_subscriptions.get_mut(channel_id) {
            subscribers.remove(user_id);
        }
    }

    /// Send message to a specific user
    pub fn send_to_user(&self, user_id: &Uuid, message: &WebSocketMessage) {
        if let Some(sender) = self.connections.get(user_id) {
            if let Ok(json) = serde_json::to_string(message) {
                match sender.send(json) {
                    Ok(n) => info!("send_to_user {}: delivered to {} receiver(s)", user_id, n),
                    Err(_) => warn!("send_to_user {}: broadcast send failed (no receivers)", user_id),
                }
            }
        } else {
            warn!("send_to_user {}: user NOT connected (not in connections map)", user_id);
        }
    }

    /// Broadcast message to all users in a channel
    pub fn broadcast_to_channel(&self, channel_id: &Uuid, message: &WebSocketMessage, exclude_user: Option<&Uuid>) {
        if let Some(subscribers) = self.channel_subscriptions.get(channel_id) {
            let json = match serde_json::to_string(message) {
                Ok(j) => j,
                Err(e) => {
                    warn!("Failed to serialize WS message: {}", e);
                    return;
                }
            };

            let subscriber_count = subscribers.len();
            let mut sent_count = 0;
            let mut no_connection_count = 0;
            let mut send_error_count = 0;

            for user_id in subscribers.iter() {
                if exclude_user.map_or(true, |excluded| excluded != user_id) {
                    if let Some(sender) = self.connections.get(user_id) {
                        match sender.send(json.clone()) {
                            Ok(_) => sent_count += 1,
                            Err(_) => send_error_count += 1,
                        }
                    } else {
                        no_connection_count += 1;
                    }
                }
            }

            debug!(
                "Broadcast to channel {}: {} subscribers, {} sent, {} no connection, {} send errors",
                channel_id, subscriber_count, sent_count, no_connection_count, send_error_count
            );
        } else {
            debug!("Broadcast to channel {}: no subscribers found", channel_id);
        }
    }

    /// Broadcast message to all connected users
    pub fn broadcast_to_all(&self, message: &WebSocketMessage, exclude_user: Option<&Uuid>) {
        let json = match serde_json::to_string(message) {
            Ok(j) => j,
            Err(_) => return,
        };

        for entry in self.connections.iter() {
            if exclude_user.map_or(true, |excluded| excluded != entry.key()) {
                let _ = entry.value().send(json.clone());
            }
        }
    }

    /// Update user status
    pub fn update_user_status(&self, user_id: &Uuid, status: UserStatus) {
        self.user_statuses.insert(*user_id, status);
    }

    /// Get user status
    pub fn get_user_status(&self, user_id: &Uuid) -> UserStatus {
        self.user_statuses.get(user_id).map(|s| s.clone()).unwrap_or(UserStatus::Offline)
    }

    /// Get online users in a channel
    pub fn get_online_users_in_channel(&self, channel_id: &Uuid) -> Vec<Uuid> {
        self.channel_subscriptions
            .get(channel_id)
            .map(|subscribers| {
                subscribers
                    .iter()
                    .filter(|user_id| self.connections.contains_key(user_id))
                    .copied()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Subscribe user to a call
    pub fn subscribe_to_call(&self, call_id: Uuid, user_id: Uuid) {
        self.call_subscriptions
            .entry(call_id)
            .or_insert_with(HashSet::new)
            .insert(user_id);
    }

    /// Unsubscribe user from a call
    pub fn unsubscribe_from_call(&self, call_id: &Uuid, user_id: &Uuid) {
        if let Some(mut participants) = self.call_subscriptions.get_mut(call_id) {
            participants.remove(user_id);
        }
    }

    /// Send message to all participants in a call
    pub fn send_to_call_participants(&self, call_id: &Uuid, message: &WebSocketMessage, exclude_user: Option<&Uuid>) {
        if let Some(participants) = self.call_subscriptions.get(call_id) {
            let json = match serde_json::to_string(message) {
                Ok(j) => j,
                Err(_) => return,
            };

            for user_id in participants.iter() {
                if exclude_user.map_or(true, |excluded| excluded != user_id) {
                    if let Some(sender) = self.connections.get(user_id) {
                        let _ = sender.send(json.clone());
                    }
                }
            }
        }
    }

    /// Get participants in a call
    pub fn get_call_participants(&self, call_id: &Uuid) -> Vec<Uuid> {
        self.call_subscriptions
            .get(call_id)
            .map(|participants| participants.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Remove call subscription entirely (when call ends)
    pub fn remove_call(&self, call_id: &Uuid) {
        self.call_subscriptions.remove(call_id);
    }
}

/// WebSocket connection handler
pub async fn ws_handler(
    req: HttpRequest,
    body: web::Payload,
    services: web::Data<Arc<Services>>,
    ws_server: web::Data<Arc<WebSocketServer>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let services = services.get_ref().clone();
    let ws_server = ws_server.get_ref().clone();

    // Spawn WebSocket handler task
    actix_rt::spawn(async move {
        let mut user_id: Option<Uuid> = None;
        let mut _receiver: Option<broadcast::Receiver<String>> = None;

        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    // Parse incoming message
                    let ws_msg: Result<WebSocketMessage, _> = serde_json::from_str(&text);

                    match ws_msg {
                        Ok(WebSocketMessage::Authenticate { token }) => {
                            // Verify token and get user
                            match services.auth.verify_access_token(&token) {
                                Ok(uid) => {
                                    info!("WebSocket authenticated: user {}", uid);
                                    user_id = Some(uid);
                                    let receiver = ws_server.register(uid);

                                    // Get user info
                                    if let Ok(user) = services.users.get_user(&uid).await {
                                        let response = WebSocketMessage::Authenticated { user };
                                        if let Ok(json) = serde_json::to_string(&response) {
                                            let _ = session.text(json).await;
                                        }
                                    }

                                    // Start listening for broadcasts
                                    let mut rx = receiver;
                                    let mut session_clone = session.clone();
                                    let uid_for_task = uid;
                                    actix_rt::spawn(async move {
                                        loop {
                                            match rx.recv().await {
                                                Ok(msg) => {
                                                    if session_clone.text(msg).await.is_err() {
                                                        info!("WebSocket session closed for user {}", uid_for_task);
                                                        break;
                                                    }
                                                }
                                                Err(broadcast::error::RecvError::Lagged(n)) => {
                                                    warn!("WebSocket receiver lagged by {} messages for user {}", n, uid_for_task);
                                                    // Continue receiving instead of disconnecting
                                                    continue;
                                                }
                                                Err(broadcast::error::RecvError::Closed) => {
                                                    info!("Broadcast channel closed for user {}", uid_for_task);
                                                    break;
                                                }
                                            }
                                        }
                                    });
                                }
                                Err(e) => {
                                    warn!("WebSocket auth failed: {}", e);
                                    let error = WebSocketMessage::Error {
                                        code: "AUTH_FAILED".to_string(),
                                        message: e.to_string(),
                                    };
                                    if let Ok(json) = serde_json::to_string(&error) {
                                        let _ = session.text(json).await;
                                    }
                                }
                            }
                        }

                        Ok(WebSocketMessage::JoinChannel { channel_id }) => {
                            if let Some(uid) = user_id {
                                // Verify user has access to channel
                                match services.channels.is_member(&channel_id, &uid).await {
                                    Ok(true) => {
                                        ws_server.subscribe_to_channel(channel_id, uid);
                                        info!("User {} subscribed to channel {}", uid, channel_id);

                                        // Notify others in channel
                                        if let Ok(user) = services.users.get_user(&uid).await {
                                            let msg = WebSocketMessage::UserJoinedChannel {
                                                channel_id,
                                                user,
                                            };
                                            ws_server.broadcast_to_channel(&channel_id, &msg, Some(&uid));
                                        }
                                    }
                                    Ok(false) => {
                                        warn!("User {} is not a member of channel {}", uid, channel_id);
                                    }
                                    Err(e) => {
                                        warn!("Failed to check membership for user {} in channel {}: {}", uid, channel_id, e);
                                    }
                                }
                            } else {
                                warn!("JoinChannel received but user is not authenticated");
                            }
                        }

                        Ok(WebSocketMessage::LeaveChannel { channel_id }) => {
                            if let Some(uid) = user_id {
                                ws_server.unsubscribe_from_channel(&channel_id, &uid);

                                // Notify others in channel
                                let msg = WebSocketMessage::UserLeftChannel {
                                    channel_id,
                                    user_id: uid,
                                };
                                ws_server.broadcast_to_channel(&channel_id, &msg, Some(&uid));
                            }
                        }

                        Ok(WebSocketMessage::SendMessage { channel_id, content, reply_to_id }) => {
                            if let Some(uid) = user_id {
                                // Send message through service
                                let request = shared::dto::SendMessageRequest {
                                    content,
                                    message_type: None,
                                    reply_to_id,
                                    attachment_ids: None,
                                };

                                if let Ok(message) = services.messages.send_message(&channel_id, &uid, request).await {
                                    // Broadcast to channel
                                    let msg = WebSocketMessage::NewMessage { message };
                                    ws_server.broadcast_to_channel(&channel_id, &msg, None);
                                }
                            }
                        }

                        Ok(WebSocketMessage::StartTyping { channel_id }) => {
                            if let Some(uid) = user_id {
                                if let Ok(user) = services.users.get_user(&uid).await {
                                    let msg = WebSocketMessage::UserTyping {
                                        channel_id,
                                        user,
                                    };
                                    ws_server.broadcast_to_channel(&channel_id, &msg, Some(&uid));
                                }
                            }
                        }

                        Ok(WebSocketMessage::StopTyping { channel_id }) => {
                            if let Some(uid) = user_id {
                                let msg = WebSocketMessage::UserStoppedTyping {
                                    channel_id,
                                    user_id: uid,
                                };
                                ws_server.broadcast_to_channel(&channel_id, &msg, Some(&uid));
                            }
                        }

                        Ok(WebSocketMessage::UpdateStatus { status, status_message }) => {
                            if let Some(uid) = user_id {
                                // Update status in database
                                let _ = services.users.update_status(&uid, status, status_message.as_deref()).await;
                                ws_server.update_user_status(&uid, status);

                                // Broadcast status change
                                let msg = WebSocketMessage::UserStatusChanged {
                                    user_id: uid,
                                    status,
                                    status_message,
                                };
                                ws_server.broadcast_to_all(&msg, Some(&uid));
                            }
                        }

                        // Join a call (WebSocket subscription for signaling)
                        Ok(WebSocketMessage::JoinCall { call_id }) => {
                            if let Some(uid) = user_id {
                                ws_server.subscribe_to_call(call_id, uid);
                                info!("User {} joined call {}", uid, call_id);
                            }
                        }

                        // Leave a call
                        Ok(WebSocketMessage::LeaveCall { call_id }) => {
                            if let Some(uid) = user_id {
                                ws_server.unsubscribe_from_call(&call_id, &uid);

                                // Notify other participants
                                let msg = WebSocketMessage::ParticipantLeft {
                                    call_id,
                                    user_id: uid,
                                };
                                ws_server.send_to_call_participants(&call_id, &msg, Some(&uid));
                                info!("User {} left call {}", uid, call_id);
                            }
                        }

                        // WebRTC Offer - Forward to all participants except sender
                        Ok(WebSocketMessage::WebRTCOffer { call_id, from_user_id: _, sdp }) => {
                            if let Some(uid) = user_id {
                                let participants = ws_server.get_call_participants(&call_id);
                                info!("WebRTC offer from {} in call {} → forwarding to {} participant(s): {:?}",
                                    uid, call_id, participants.len().saturating_sub(1),
                                    participants.iter().filter(|id| **id != uid).collect::<Vec<_>>());
                                let msg = WebSocketMessage::WebRTCOffer {
                                    call_id,
                                    from_user_id: uid,
                                    sdp,
                                };
                                ws_server.send_to_call_participants(&call_id, &msg, Some(&uid));
                            }
                        }

                        // WebRTC Answer - Forward to all participants except sender
                        Ok(WebSocketMessage::WebRTCAnswer { call_id, from_user_id: _, sdp }) => {
                            if let Some(uid) = user_id {
                                let participants = ws_server.get_call_participants(&call_id);
                                info!("WebRTC answer from {} in call {} → forwarding to {} participant(s): {:?}",
                                    uid, call_id, participants.len().saturating_sub(1),
                                    participants.iter().filter(|id| **id != uid).collect::<Vec<_>>());
                                let msg = WebSocketMessage::WebRTCAnswer {
                                    call_id,
                                    from_user_id: uid,
                                    sdp,
                                };
                                ws_server.send_to_call_participants(&call_id, &msg, Some(&uid));
                            }
                        }

                        // WebRTC ICE Candidate - Forward to all participants except sender
                        Ok(WebSocketMessage::WebRTCIceCandidate { call_id, from_user_id: _, candidate }) => {
                            if let Some(uid) = user_id {
                                let msg = WebSocketMessage::WebRTCIceCandidate {
                                    call_id,
                                    from_user_id: uid,
                                    candidate,
                                };
                                ws_server.send_to_call_participants(&call_id, &msg, Some(&uid));
                                info!("Forwarded ICE candidate from {} in call {}", uid, call_id);
                            }
                        }

                        Err(e) => {
                            // Ignore heartbeat pings sent as text
                            if &*text != "ping" {
                                warn!("Failed to parse WebSocket message: {} | raw: {}", e, text);
                            }
                        }
                        _ => {}
                    }
                }

                Message::Ping(bytes) => {
                    let _ = session.pong(&bytes).await;
                }

                Message::Close(_) => {
                    if let Some(uid) = user_id {
                        ws_server.unregister(&uid);

                        // Update user status to offline
                        let _ = services.users.update_status(&uid, UserStatus::Offline, None).await;

                        // Broadcast status change
                        let msg = WebSocketMessage::UserStatusChanged {
                            user_id: uid,
                            status: UserStatus::Offline,
                            status_message: None,
                        };
                        ws_server.broadcast_to_all(&msg, Some(&uid));
                    }
                    break;
                }

                _ => {}
            }
        }

        // Cleanup on disconnect
        if let Some(uid) = user_id {
            ws_server.unregister(&uid);
        }
    });

    Ok(response)
}
