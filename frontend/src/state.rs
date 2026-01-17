//! Application state management

use shared::dto::{ChannelResponse, MessageResponse, TeamResponse, UserResponse};
use shared::models::UserStatus;
use std::collections::HashMap;
use uuid::Uuid;

/// Global application state
#[derive(Clone, Default)]
pub struct AppState {
    /// Current authenticated user
    pub current_user: Option<UserResponse>,
    /// Access token
    pub access_token: Option<String>,
    /// Refresh token
    pub refresh_token: Option<String>,
    /// User's teams
    pub teams: Vec<TeamResponse>,
    /// User's channels
    pub channels: Vec<ChannelResponse>,
    /// Currently selected team
    pub selected_team_id: Option<Uuid>,
    /// Currently selected channel
    pub selected_channel_id: Option<Uuid>,
    /// Messages by channel ID
    pub messages: HashMap<Uuid, Vec<MessageResponse>>,
    /// Online users by user ID
    pub online_users: HashMap<Uuid, UserStatus>,
    /// Typing users by channel ID
    pub typing_users: HashMap<Uuid, Vec<UserResponse>>,
    /// Unread counts by channel ID
    pub unread_counts: HashMap<Uuid, i64>,
    /// WebSocket connected
    pub ws_connected: bool,
    /// Loading state
    pub loading: bool,
    /// Error message
    pub error: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_authenticated(&self) -> bool {
        self.current_user.is_some() && self.access_token.is_some()
    }

    pub fn set_auth(&mut self, user: UserResponse, access_token: String, refresh_token: String) {
        self.current_user = Some(user);
        self.access_token = Some(access_token);
        self.refresh_token = Some(refresh_token);
    }

    pub fn clear_auth(&mut self) {
        self.current_user = None;
        self.access_token = None;
        self.refresh_token = None;
        self.teams.clear();
        self.channels.clear();
        self.messages.clear();
        self.selected_team_id = None;
        self.selected_channel_id = None;
    }

    pub fn logout(&mut self) {
        self.clear_auth();
        // Clear from local storage
        if let Ok(Some(storage)) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .map(|s| s)
        {
            let _ = storage.remove_item("access_token");
            let _ = storage.remove_item("refresh_token");
        }
    }

    pub fn set_teams(&mut self, teams: Vec<TeamResponse>) {
        self.teams = teams;
    }

    pub fn set_channels(&mut self, channels: Vec<ChannelResponse>) {
        self.channels = channels;
    }

    pub fn add_message(&mut self, channel_id: Uuid, message: MessageResponse) {
        self.messages
            .entry(channel_id)
            .or_insert_with(Vec::new)
            .push(message);
    }

    pub fn set_messages(&mut self, channel_id: Uuid, messages: Vec<MessageResponse>) {
        self.messages.insert(channel_id, messages);
    }

    pub fn update_message(&mut self, channel_id: Uuid, message: MessageResponse) {
        if let Some(messages) = self.messages.get_mut(&channel_id) {
            if let Some(pos) = messages.iter().position(|m| m.id == message.id) {
                messages[pos] = message;
            }
        }
    }

    pub fn delete_message(&mut self, channel_id: Uuid, message_id: Uuid) {
        if let Some(messages) = self.messages.get_mut(&channel_id) {
            messages.retain(|m| m.id != message_id);
        }
    }

    pub fn set_user_status(&mut self, user_id: Uuid, status: UserStatus) {
        self.online_users.insert(user_id, status);
    }

    pub fn add_typing_user(&mut self, channel_id: Uuid, user: UserResponse) {
        let users = self.typing_users.entry(channel_id).or_insert_with(Vec::new);
        if !users.iter().any(|u| u.id == user.id) {
            users.push(user);
        }
    }

    pub fn remove_typing_user(&mut self, channel_id: Uuid, user_id: Uuid) {
        if let Some(users) = self.typing_users.get_mut(&channel_id) {
            users.retain(|u| u.id != user_id);
        }
    }

    pub fn set_unread_count(&mut self, channel_id: Uuid, count: i64) {
        self.unread_counts.insert(channel_id, count);
    }

    pub fn clear_unread(&mut self, channel_id: Uuid) {
        self.unread_counts.remove(&channel_id);
    }

    pub fn get_channel(&self, channel_id: &Uuid) -> Option<&ChannelResponse> {
        self.channels.iter().find(|c| &c.id == channel_id)
    }

    pub fn get_team(&self, team_id: &Uuid) -> Option<&TeamResponse> {
        self.teams.iter().find(|t| &t.id == team_id)
    }

    pub fn get_messages(&self, channel_id: &Uuid) -> Option<&Vec<MessageResponse>> {
        self.messages.get(channel_id)
    }
}
