//! Channel service

use shared::dto::{
    ChannelMemberResponse, ChannelResponse, CreateChannelRequest, MessageResponse,
    UpdateChannelRequest, UserResponse,
};
use shared::error::AppError;
use shared::models::ChannelType;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::{ChannelRepository, MessageRepository, TeamRepository, UserRepository};

pub struct ChannelService {
    pool: Arc<PgPool>,
}

impl ChannelService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_channel(
        &self,
        user_id: &Uuid,
        request: CreateChannelRequest,
    ) -> Result<ChannelResponse, AppError> {
        // If team_id is provided, verify user is a team member
        if let Some(team_id) = &request.team_id {
            if !TeamRepository::is_member(&self.pool, team_id, user_id)
                .await
                .unwrap_or(false)
            {
                return Err(AppError::AuthorizationError(
                    "You are not a member of this team".to_string(),
                ));
            }
        }

        let channel_type = request.channel_type.unwrap_or(ChannelType::Public);

        let channel = ChannelRepository::create(
            &self.pool,
            request.team_id.as_ref(),
            &request.name,
            request.description.as_deref(),
            channel_type,
            user_id,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Add additional members if specified
        if let Some(member_ids) = request.member_ids {
            for member_id in member_ids {
                if member_id != *user_id {
                    let _ = ChannelRepository::add_member(&self.pool, &channel.id, &member_id).await;
                }
            }
        }

        self.get_channel_response(&channel.id, user_id).await
    }

    pub async fn create_dm_channel(
        &self,
        user_id: &Uuid,
        other_user_id: &Uuid,
    ) -> Result<ChannelResponse, AppError> {
        // Check if DM channel already exists
        if let Some(channel) = ChannelRepository::find_dm_channel(&self.pool, user_id, other_user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
        {
            return self.get_channel_response(&channel.id, user_id).await;
        }

        // Get other user's display name for channel name
        let other_user = UserRepository::find_by_id(&self.pool, other_user_id)
            .await
            .map_err(|_| AppError::NotFoundError("User not found".to_string()))?;

        let channel = ChannelRepository::create(
            &self.pool,
            None,
            &format!("DM: {}", other_user.display_name),
            None,
            ChannelType::DirectMessage,
            user_id,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Add other user to channel
        ChannelRepository::add_member(&self.pool, &channel.id, other_user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        self.get_channel_response(&channel.id, user_id).await
    }

    pub async fn get_channel(
        &self,
        channel_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<ChannelResponse, AppError> {
        // Verify user has access
        self.check_channel_access(channel_id, user_id).await?;
        self.get_channel_response(channel_id, user_id).await
    }

    pub async fn list_user_channels(&self, user_id: &Uuid) -> Result<Vec<ChannelResponse>, AppError> {
        let channels = ChannelRepository::find_by_user(&self.pool, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut responses = Vec::new();
        for channel in channels {
            if let Ok(response) = self.get_channel_response(&channel.id, user_id).await {
                responses.push(response);
            }
        }

        Ok(responses)
    }

    pub async fn list_team_channels(
        &self,
        team_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<ChannelResponse>, AppError> {
        // Verify user is a team member
        if !TeamRepository::is_member(&self.pool, team_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You are not a member of this team".to_string(),
            ));
        }

        let channels = ChannelRepository::find_by_team(&self.pool, team_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut responses = Vec::new();
        for channel in channels {
            if let Ok(response) = self.get_channel_response(&channel.id, user_id).await {
                responses.push(response);
            }
        }

        Ok(responses)
    }

    pub async fn update_channel(
        &self,
        channel_id: &Uuid,
        user_id: &Uuid,
        request: UpdateChannelRequest,
    ) -> Result<ChannelResponse, AppError> {
        self.check_channel_access(channel_id, user_id).await?;

        let channel = ChannelRepository::find_by_id(&self.pool, channel_id)
            .await
            .map_err(|_| AppError::NotFoundError("Channel not found".to_string()))?;

        // Only creator can update channel
        if channel.created_by != *user_id {
            return Err(AppError::AuthorizationError(
                "Only the channel creator can update it".to_string(),
            ));
        }

        ChannelRepository::update(
            &self.pool,
            channel_id,
            request.name.as_deref(),
            request.description.as_deref(),
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        self.get_channel_response(channel_id, user_id).await
    }

    pub async fn delete_channel(&self, channel_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        let channel = ChannelRepository::find_by_id(&self.pool, channel_id)
            .await
            .map_err(|_| AppError::NotFoundError("Channel not found".to_string()))?;

        // Only creator can delete channel
        if channel.created_by != *user_id {
            return Err(AppError::AuthorizationError(
                "Only the channel creator can delete it".to_string(),
            ));
        }

        ChannelRepository::delete(&self.pool, channel_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_members(
        &self,
        channel_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<ChannelMemberResponse>, AppError> {
        self.check_channel_access(channel_id, user_id).await?;

        let members = ChannelRepository::find_members(&self.pool, channel_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut responses = Vec::new();
        for member in members {
            let user = UserRepository::find_by_id(&self.pool, &member.user_id)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            responses.push(ChannelMemberResponse {
                id: member.id,
                user: UserResponse {
                    id: user.id,
                    email: user.email,
                    username: user.username,
                    display_name: user.display_name,
                    avatar_url: user.avatar_url,
                    status: user.status,
                    status_message: user.status_message,
                    last_seen: user.last_seen,
                    created_at: user.created_at,
                },
                joined_at: member.joined_at,
                last_read_at: member.last_read_at,
            });
        }

        Ok(responses)
    }

    pub async fn add_member(
        &self,
        channel_id: &Uuid,
        requester_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<ChannelMemberResponse, AppError> {
        self.check_channel_access(channel_id, requester_id).await?;

        let channel = ChannelRepository::find_by_id(&self.pool, channel_id)
            .await
            .map_err(|_| AppError::NotFoundError("Channel not found".to_string()))?;

        // Cannot add members to DM channels
        if channel.channel_type == ChannelType::DirectMessage {
            return Err(AppError::BadRequest(
                "Cannot add members to direct message channels".to_string(),
            ));
        }

        // Check if already a member
        if ChannelRepository::is_member(&self.pool, channel_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::ConflictError("User is already a channel member".to_string()));
        }

        let member = ChannelRepository::add_member(&self.pool, channel_id, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let user = UserRepository::find_by_id(&self.pool, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(ChannelMemberResponse {
            id: member.id,
            user: UserResponse {
                id: user.id,
                email: user.email,
                username: user.username,
                display_name: user.display_name,
                avatar_url: user.avatar_url,
                status: user.status,
                status_message: user.status_message,
                last_seen: user.last_seen,
                created_at: user.created_at,
            },
            joined_at: member.joined_at,
            last_read_at: member.last_read_at,
        })
    }

    pub async fn remove_member(
        &self,
        channel_id: &Uuid,
        requester_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), AppError> {
        let channel = ChannelRepository::find_by_id(&self.pool, channel_id)
            .await
            .map_err(|_| AppError::NotFoundError("Channel not found".to_string()))?;

        // Cannot remove from DM channels
        if channel.channel_type == ChannelType::DirectMessage {
            return Err(AppError::BadRequest(
                "Cannot remove members from direct message channels".to_string(),
            ));
        }

        // Can remove self or if you're the creator
        if requester_id != user_id && channel.created_by != *requester_id {
            return Err(AppError::AuthorizationError(
                "You don't have permission to remove this member".to_string(),
            ));
        }

        ChannelRepository::remove_member(&self.pool, channel_id, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn mark_as_read(&self, channel_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        self.check_channel_access(channel_id, user_id).await?;

        ChannelRepository::mark_as_read(&self.pool, channel_id, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn is_member(&self, channel_id: &Uuid, user_id: &Uuid) -> Result<bool, AppError> {
        ChannelRepository::is_member(&self.pool, channel_id, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    async fn check_channel_access(&self, channel_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        let channel = ChannelRepository::find_by_id(&self.pool, channel_id)
            .await
            .map_err(|_| AppError::NotFoundError("Channel not found".to_string()))?;

        // For public channels in a team, check team membership
        if channel.channel_type == ChannelType::Public {
            if let Some(team_id) = channel.team_id {
                if TeamRepository::is_member(&self.pool, &team_id, user_id)
                    .await
                    .unwrap_or(false)
                {
                    return Ok(());
                }
            }
        }

        // For private channels and DMs, check channel membership
        if ChannelRepository::is_member(&self.pool, channel_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Ok(());
        }

        Err(AppError::AuthorizationError(
            "You don't have access to this channel".to_string(),
        ))
    }

    async fn get_channel_response(
        &self,
        channel_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<ChannelResponse, AppError> {
        let channel = ChannelRepository::find_by_id(&self.pool, channel_id)
            .await
            .map_err(|_| AppError::NotFoundError("Channel not found".to_string()))?;

        let member_count = ChannelRepository::get_member_count(&self.pool, channel_id)
            .await
            .unwrap_or(0);

        let unread_count = ChannelRepository::get_unread_count(&self.pool, channel_id, user_id)
            .await
            .unwrap_or(0);

        let last_message = MessageRepository::get_last_message(&self.pool, channel_id)
            .await
            .ok()
            .flatten();

        let last_message_response = if let Some(msg) = last_message {
            let sender = UserRepository::find_by_id(&self.pool, &msg.sender_id)
                .await
                .ok();

            sender.map(|s| MessageResponse {
                id: msg.id,
                channel_id: msg.channel_id,
                sender: UserResponse {
                    id: s.id,
                    email: s.email,
                    username: s.username,
                    display_name: s.display_name,
                    avatar_url: s.avatar_url,
                    status: s.status,
                    status_message: s.status_message,
                    last_seen: s.last_seen,
                    created_at: s.created_at,
                },
                content: msg.content,
                message_type: msg.message_type,
                reply_to: None,
                reactions: vec![],
                attachments: vec![],
                edited: msg.edited,
                created_at: msg.created_at,
                updated_at: msg.updated_at,
            })
        } else {
            None
        };

        Ok(ChannelResponse {
            id: channel.id,
            team_id: channel.team_id,
            name: channel.name,
            description: channel.description,
            channel_type: channel.channel_type,
            member_count,
            unread_count,
            last_message: last_message_response,
            created_at: channel.created_at,
        })
    }
}
