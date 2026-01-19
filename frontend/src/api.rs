//! API client for backend communication

use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{de::DeserializeOwned, Serialize};
use shared::dto::*;
use shared::error::ErrorResponse;

const API_BASE_URL: &str = "/api/v1";
const TOKEN_KEY: &str = "rust_teams_token";
const REFRESH_TOKEN_KEY: &str = "rust_teams_refresh_token";

pub struct ApiClient;

impl ApiClient {
    fn get_token() -> Option<String> {
        LocalStorage::get::<String>(TOKEN_KEY).ok()
    }

    fn set_tokens(access_token: &str, refresh_token: &str) {
        let _ = LocalStorage::set(TOKEN_KEY, access_token);
        let _ = LocalStorage::set(REFRESH_TOKEN_KEY, refresh_token);
    }

    fn clear_tokens() {
        LocalStorage::delete(TOKEN_KEY);
        LocalStorage::delete(REFRESH_TOKEN_KEY);
    }

    async fn request<T: DeserializeOwned>(
        method: &str,
        path: &str,
        body: Option<impl Serialize>,
        auth: bool,
    ) -> Result<T, String> {
        let url = format!("{}{}", API_BASE_URL, path);

        let request = match method {
            "GET" => Request::get(&url),
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url),
            "PATCH" => Request::patch(&url),
            "DELETE" => Request::delete(&url),
            _ => return Err("Invalid HTTP method".to_string()),
        };

        let mut request = if auth {
            if let Some(token) = Self::get_token() {
                request.header("Authorization", &format!("Bearer {}", token))
            } else {
                request
            }
        } else {
            request
        };

        let response = if let Some(body) = body {
            request
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&body).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?
        } else {
            request.send().await.map_err(|e| e.to_string())?
        };

        if response.ok() {
            response.json::<T>().await.map_err(|e| e.to_string())
        } else {
            let error: ErrorResponse = response.json().await.map_err(|e| e.to_string())?;
            Err(error.error.message)
        }
    }

    // Authentication
    pub async fn register(request: RegisterRequest) -> Result<AuthResponse, String> {
        let response: AuthResponse = Self::request("POST", "/auth/register", Some(request), false).await?;
        Self::set_tokens(&response.access_token, &response.refresh_token);
        Ok(response)
    }

    pub async fn login(request: LoginRequest) -> Result<AuthResponse, String> {
        let response: AuthResponse = Self::request("POST", "/auth/login", Some(request), false).await?;
        Self::set_tokens(&response.access_token, &response.refresh_token);
        Ok(response)
    }

    pub async fn logout() {
        let _ = Self::request::<serde_json::Value>("POST", "/auth/logout", None::<()>, true).await;
        Self::clear_tokens();
    }

    pub async fn refresh_token() -> Result<AuthResponse, String> {
        let refresh_token: String = LocalStorage::get(REFRESH_TOKEN_KEY).map_err(|_| "No refresh token")?;
        let request = RefreshTokenRequest { refresh_token };
        let response: AuthResponse = Self::request("POST", "/auth/refresh", Some(request), false).await?;
        Self::set_tokens(&response.access_token, &response.refresh_token);
        Ok(response)
    }

    // Users
    pub async fn get_current_user() -> Result<UserResponse, String> {
        Self::request("GET", "/users/me", None::<()>, true).await
    }

    pub async fn update_current_user(request: UpdateUserRequest) -> Result<UserResponse, String> {
        Self::request("PATCH", "/users/me", Some(request), true).await
    }

    pub async fn change_password(request: ChangePasswordRequest) -> Result<(), String> {
        Self::request::<serde_json::Value>("PUT", "/users/me/password", Some(request), true).await?;
        Ok(())
    }

    pub async fn search_users(query: &str) -> Result<Vec<UserResponse>, String> {
        Self::request("GET", &format!("/users/search?q={}", query), None::<()>, true).await
    }

    // Teams
    pub async fn list_teams() -> Result<Vec<TeamResponse>, String> {
        Self::request("GET", "/teams", None::<()>, true).await
    }

    pub async fn create_team(request: CreateTeamRequest) -> Result<TeamResponse, String> {
        Self::request("POST", "/teams", Some(request), true).await
    }

    pub async fn get_team(team_id: &str) -> Result<TeamResponse, String> {
        Self::request("GET", &format!("/teams/{}", team_id), None::<()>, true).await
    }

    pub async fn update_team(team_id: &str, request: UpdateTeamRequest) -> Result<TeamResponse, String> {
        Self::request("PATCH", &format!("/teams/{}", team_id), Some(request), true).await
    }

    pub async fn delete_team(team_id: &str) -> Result<(), String> {
        Self::request::<serde_json::Value>("DELETE", &format!("/teams/{}", team_id), None::<()>, true).await?;
        Ok(())
    }

    pub async fn list_team_members(team_id: &str) -> Result<Vec<TeamMemberResponse>, String> {
        Self::request("GET", &format!("/teams/{}/members", team_id), None::<()>, true).await
    }

    // Channels
    pub async fn list_channels() -> Result<Vec<ChannelResponse>, String> {
        Self::request("GET", "/channels", None::<()>, true).await
    }

    pub async fn list_team_channels(team_id: &str) -> Result<Vec<ChannelResponse>, String> {
        Self::request("GET", &format!("/teams/{}/channels", team_id), None::<()>, true).await
    }

    pub async fn create_channel(request: CreateChannelRequest) -> Result<ChannelResponse, String> {
        Self::request("POST", "/channels", Some(request), true).await
    }

    pub async fn get_channel(channel_id: &str) -> Result<ChannelResponse, String> {
        Self::request("GET", &format!("/channels/{}", channel_id), None::<()>, true).await
    }

    pub async fn list_channel_members(channel_id: &str) -> Result<Vec<ChannelMemberResponse>, String> {
        Self::request("GET", &format!("/channels/{}/members", channel_id), None::<()>, true).await
    }

    pub async fn mark_channel_as_read(channel_id: &str) -> Result<(), String> {
        Self::request::<serde_json::Value>("POST", &format!("/channels/{}/read", channel_id), None::<()>, true).await?;
        Ok(())
    }

    // Messages
    pub async fn list_messages(channel_id: &str, limit: Option<i64>) -> Result<Vec<MessageResponse>, String> {
        let limit = limit.unwrap_or(50);
        Self::request("GET", &format!("/channels/{}/messages?limit={}", channel_id, limit), None::<()>, true).await
    }

    pub async fn send_message(channel_id: &str, request: SendMessageRequest) -> Result<MessageResponse, String> {
        Self::request("POST", &format!("/channels/{}/messages", channel_id), Some(request), true).await
    }

    pub async fn update_message(channel_id: &str, message_id: &str, request: UpdateMessageRequest) -> Result<MessageResponse, String> {
        Self::request("PATCH", &format!("/channels/{}/messages/{}", channel_id, message_id), Some(request), true).await
    }

    pub async fn delete_message(channel_id: &str, message_id: &str) -> Result<(), String> {
        Self::request::<serde_json::Value>("DELETE", &format!("/channels/{}/messages/{}", channel_id, message_id), None::<()>, true).await?;
        Ok(())
    }

    pub async fn add_reaction(channel_id: &str, message_id: &str, emoji: &str) -> Result<(), String> {
        let request = AddReactionRequest { emoji: emoji.to_string() };
        Self::request::<serde_json::Value>("POST", &format!("/channels/{}/messages/{}/reactions", channel_id, message_id), Some(request), true).await?;
        Ok(())
    }

    pub async fn remove_reaction(channel_id: &str, message_id: &str, emoji: &str) -> Result<(), String> {
        Self::request::<serde_json::Value>("DELETE", &format!("/channels/{}/messages/{}/reactions/{}", channel_id, message_id, emoji), None::<()>, true).await?;
        Ok(())
    }

    // Calls
    pub async fn start_call(request: StartCallRequest) -> Result<CallResponse, String> {
        Self::request("POST", "/calls", Some(request), true).await
    }

    pub async fn join_call(call_id: &str) -> Result<CallResponse, String> {
        Self::request("POST", &format!("/calls/{}/join", call_id), None::<()>, true).await
    }

    pub async fn leave_call(call_id: &str) -> Result<(), String> {
        Self::request::<serde_json::Value>("POST", &format!("/calls/{}/leave", call_id), None::<()>, true).await?;
        Ok(())
    }

    pub async fn end_call(call_id: &str) -> Result<(), String> {
        Self::request::<serde_json::Value>("POST", &format!("/calls/{}/end", call_id), None::<()>, true).await?;
        Ok(())
    }

    // Search
    pub async fn search_messages(query: &str) -> Result<SearchResponse, String> {
        Self::request("GET", &format!("/search/messages?q={}", query), None::<()>, true).await
    }

    // Notifications
    pub async fn list_notifications() -> Result<Vec<NotificationResponse>, String> {
        Self::request("GET", "/notifications", None::<()>, true).await
    }

    pub async fn mark_notification_as_read(notification_id: &str) -> Result<(), String> {
        Self::request::<serde_json::Value>("POST", &format!("/notifications/{}/read", notification_id), None::<()>, true).await?;
        Ok(())
    }

    pub async fn mark_all_notifications_as_read() -> Result<(), String> {
        Self::request::<serde_json::Value>("POST", "/notifications/read-all", None::<()>, true).await?;
        Ok(())
    }
}
