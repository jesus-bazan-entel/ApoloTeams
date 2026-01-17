//! Domain models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Online,
    Available,
    Away,
    Busy,
    DoNotDisturb,
    Offline,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Offline
    }
}

/// User model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub status_message: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Team member role
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TeamRole {
    Owner,
    Admin,
    Member,
}

impl Default for TeamRole {
    fn default() -> Self {
        Self::Member
    }
}

/// Channel type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Public,
    Private,
    DirectMessage,
}

impl Default for ChannelType {
    fn default() -> Self {
        Self::Public
    }
}

/// Message type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Image,
    File,
    System,
}

impl Default for MessageType {
    fn default() -> Self {
        Self::Text
    }
}

/// Call type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CallType {
    Audio,
    Video,
}

/// Call status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CallStatus {
    Ringing,
    Active,
    InProgress,
    Ended,
    Missed,
    Declined,
}

/// Notification type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    Message,
    Mention,
    Call,
    TeamInvite,
    System,
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::Message => write!(f, "message"),
            NotificationType::Mention => write!(f, "mention"),
            NotificationType::Call => write!(f, "call"),
            NotificationType::TeamInvite => write!(f, "team_invite"),
            NotificationType::System => write!(f, "system"),
        }
    }
}

impl std::str::FromStr for NotificationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "message" => Ok(NotificationType::Message),
            "mention" => Ok(NotificationType::Mention),
            "call" => Ok(NotificationType::Call),
            "team_invite" | "teaminvite" => Ok(NotificationType::TeamInvite),
            "system" => Ok(NotificationType::System),
            _ => Err(format!("Unknown notification type: {}", s)),
        }
    }
}

/// Team model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Team member model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TeamMember {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: TeamRole,
    pub joined_at: DateTime<Utc>,
}

/// Channel model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Channel {
    pub id: Uuid,
    pub team_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: ChannelType,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Channel member model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChannelMember {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
}

/// Message model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub message_type: MessageType,
    pub reply_to_id: Option<Uuid>,
    pub edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Reaction model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reaction {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

/// File attachment model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileAttachment {
    pub id: Uuid,
    pub message_id: Option<Uuid>,
    pub channel_id: Uuid,
    pub uploader_id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

/// Call model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Call {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub initiator_id: Uuid,
    pub call_type: CallType,
    pub status: CallStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

/// Call participant model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallParticipant {
    pub id: Uuid,
    pub call_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub is_muted: bool,
    pub is_video_enabled: bool,
}

/// Notification model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub body: String,
    pub notification_type: NotificationType,
    pub reference_id: Option<String>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

/// Refresh token model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
