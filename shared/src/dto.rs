//! Data Transfer Objects (DTOs) for API requests and responses
//! 
//! These structures are used for serializing/deserializing API payloads.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    CallStatus, CallType, ChannelType, MessageType, TeamRole, UserStatus,
};

// ============================================================================
// Authentication DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 3, max = 30, message = "Username must be 3-30 characters"))]
    pub username: String,
    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

// ============================================================================
// User DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub status_message: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: Option<UserStatus>,
    #[validate(length(max = 200, message = "Status message must be at most 200 characters"))]
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub new_password: String,
}

// ============================================================================
// Team DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq, Default)]
pub struct CreateTeamRequest {
    #[validate(length(min = 1, max = 100, message = "Team name must be 1-100 characters"))]
    pub name: String,
    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub owner_id: Uuid,
    pub member_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct UpdateTeamRequest {
    #[validate(length(min = 1, max = 100, message = "Team name must be 1-100 characters"))]
    pub name: Option<String>,
    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TeamMemberResponse {
    pub id: Uuid,
    pub user: UserResponse,
    pub role: TeamRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddTeamMemberRequest {
    pub user_id: Uuid,
    pub role: Option<TeamRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateTeamMemberRequest {
    pub role: TeamRole,
}

// ============================================================================
// Channel DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq, Default)]
pub struct CreateChannelRequest {
    pub team_id: Option<Uuid>,
    #[validate(length(min = 1, max = 100, message = "Channel name must be 1-100 characters"))]
    pub name: String,
    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,
    pub channel_type: Option<ChannelType>,
    /// For direct messages, list of user IDs to include
    pub member_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChannelResponse {
    pub id: Uuid,
    pub team_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: ChannelType,
    pub member_count: i64,
    pub unread_count: i64,
    pub last_message: Option<MessageResponse>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct UpdateChannelRequest {
    #[validate(length(min = 1, max = 100, message = "Channel name must be 1-100 characters"))]
    pub name: Option<String>,
    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChannelMemberResponse {
    pub id: Uuid,
    pub user: UserResponse,
    pub joined_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Message DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq, Default)]
pub struct SendMessageRequest {
    #[validate(length(min = 1, max = 10000, message = "Message must be 1-10000 characters"))]
    pub content: String,
    pub message_type: Option<MessageType>,
    pub reply_to_id: Option<Uuid>,
    pub attachment_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender: UserResponse,
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<Box<MessageResponse>>,
    pub reactions: Vec<ReactionResponse>,
    pub attachments: Vec<FileAttachmentResponse>,
    pub edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct UpdateMessageRequest {
    #[validate(length(min = 1, max = 10000, message = "Message must be 1-10000 characters"))]
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReactionResponse {
    pub emoji: String,
    pub count: i64,
    pub users: Vec<Uuid>,
    pub reacted_by_me: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddReactionRequest {
    pub emoji: String,
}

// ============================================================================
// File DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileAttachmentResponse {
    pub id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: String,
    pub download_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UploadFileResponse {
    pub id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: String,
}

// ============================================================================
// Call DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartCallRequest {
    pub channel_id: Uuid,
    pub call_type: CallType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub initiator: UserResponse,
    pub call_type: CallType,
    pub status: CallStatus,
    pub participants: Vec<CallParticipantResponse>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallParticipantResponse {
    pub user: UserResponse,
    pub joined_at: DateTime<Utc>,
    pub is_muted: bool,
    pub is_video_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateCallParticipantRequest {
    pub is_muted: Option<bool>,
    pub is_video_enabled: Option<bool>,
}

// ============================================================================
// WebSocket DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum WebSocketMessage {
    // Client -> Server
    Authenticate { token: String },
    JoinChannel { channel_id: Uuid },
    LeaveChannel { channel_id: Uuid },
    SendMessage { channel_id: Uuid, content: String, reply_to_id: Option<Uuid> },
    StartTyping { channel_id: Uuid },
    StopTyping { channel_id: Uuid },
    UpdateStatus { status: UserStatus, status_message: Option<String> },
    
    // Server -> Client
    Authenticated { user: UserResponse },
    Error { code: String, message: String },
    NewMessage { message: MessageResponse },
    MessageUpdated { message: MessageResponse },
    MessageDeleted { channel_id: Uuid, message_id: Uuid },
    UserTyping { channel_id: Uuid, user: UserResponse },
    UserStoppedTyping { channel_id: Uuid, user_id: Uuid },
    UserStatusChanged { user_id: Uuid, status: UserStatus, status_message: Option<String> },
    UserJoinedChannel { channel_id: Uuid, user: UserResponse },
    UserLeftChannel { channel_id: Uuid, user_id: Uuid },
    CallStarted { call: CallResponse },
    CallEnded { call_id: Uuid },
    ParticipantJoined { call_id: Uuid, participant: CallParticipantResponse },
    ParticipantLeft { call_id: Uuid, user_id: Uuid },
    Notification { notification: NotificationResponse },
    
    // WebRTC Signaling
    WebRTCOffer { call_id: Uuid, from_user_id: Uuid, sdp: String },
    WebRTCAnswer { call_id: Uuid, from_user_id: Uuid, sdp: String },
    WebRTCIceCandidate { call_id: Uuid, from_user_id: Uuid, candidate: String },
}

// ============================================================================
// Notification DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub notification_type: String,
    pub reference_id: Option<Uuid>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Search DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct SearchRequest {
    #[validate(length(min = 1, max = 200, message = "Query must be 1-200 characters"))]
    pub query: String,
    pub team_id: Option<Uuid>,
    pub channel_id: Option<Uuid>,
    pub from_user_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResponse {
    pub messages: Vec<MessageResponse>,
    pub total_count: i64,
}

// ============================================================================
// Pagination DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub before: Option<DateTime<Utc>>,
    pub after: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginatedResponse<T: PartialEq> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub has_more: bool,
}
