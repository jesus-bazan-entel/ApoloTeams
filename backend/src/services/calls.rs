//! Call service

use shared::dto::{CallParticipantResponse, CallResponse, UserResponse};
use shared::error::AppError;
use shared::models::{CallStatus, CallType, ChannelType};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::{CallRepository, ChannelRepository, UserRepository};

pub struct CallService {
    pool: Arc<PgPool>,
}

impl CallService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn start_call(
        &self,
        channel_id: &Uuid,
        initiator_id: &Uuid,
        call_type: CallType,
    ) -> Result<CallResponse, AppError> {
        // Verify user has access to channel
        if !ChannelRepository::is_member(&self.pool, channel_id, initiator_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this channel".to_string(),
            ));
        }

        // Check if there's already an active call in this channel
        if let Some(_) = CallRepository::find_active_by_channel(&self.pool, channel_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
        {
            return Err(AppError::ConflictError(
                "There's already an active call in this channel".to_string(),
            ));
        }

        let call = CallRepository::create(&self.pool, channel_id, initiator_id, call_type)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        self.get_call_response(&call.id).await
    }

    /// Start a direct call to another user (creates DM channel if needed)
    pub async fn start_direct_call(
        &self,
        initiator_id: &Uuid,
        target_user_id: &Uuid,
        call_type: CallType,
    ) -> Result<CallResponse, AppError> {
        // Verify target user exists
        UserRepository::find_by_id(&self.pool, target_user_id)
            .await
            .map_err(|_| AppError::NotFoundError("Target user not found".to_string()))?;

        // Find or create DM channel between the two users
        let channel = match ChannelRepository::find_dm_channel(&self.pool, initiator_id, target_user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
        {
            Some(ch) => ch,
            None => {
                // Create DM channel
                let other_user = UserRepository::find_by_id(&self.pool, target_user_id)
                    .await
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

                let channel = ChannelRepository::create(
                    &self.pool,
                    None,
                    &format!("DM: {}", other_user.display_name),
                    None,
                    ChannelType::DirectMessage,
                    initiator_id,
                )
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

                // Add target user to channel
                ChannelRepository::add_member(&self.pool, &channel.id, target_user_id)
                    .await
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

                channel
            }
        };

        // Check if there's already an active call in this channel
        if let Some(_) = CallRepository::find_active_by_channel(&self.pool, &channel.id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
        {
            return Err(AppError::ConflictError(
                "There's already an active call with this user".to_string(),
            ));
        }

        // Create the call
        let call = CallRepository::create(&self.pool, &channel.id, initiator_id, call_type)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        self.get_call_response(&call.id).await
    }

    pub async fn get_call(&self, call_id: &Uuid, user_id: &Uuid) -> Result<CallResponse, AppError> {
        let call = CallRepository::find_by_id(&self.pool, call_id)
            .await
            .map_err(|_| AppError::NotFoundError("Call not found".to_string()))?;

        // Verify user has access to channel
        if !ChannelRepository::is_member(&self.pool, &call.channel_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this call".to_string(),
            ));
        }

        self.get_call_response(call_id).await
    }

    pub async fn join_call(&self, call_id: &Uuid, user_id: &Uuid) -> Result<CallResponse, AppError> {
        let call = CallRepository::find_by_id(&self.pool, call_id)
            .await
            .map_err(|_| AppError::NotFoundError("Call not found".to_string()))?;

        // Verify user has access to channel
        if !ChannelRepository::is_member(&self.pool, &call.channel_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this call".to_string(),
            ));
        }

        // Check if call is still active
        if call.status != CallStatus::Ringing && call.status != CallStatus::InProgress {
            return Err(AppError::BadRequest("Call is no longer active".to_string()));
        }

        // Check if already a participant
        if CallRepository::is_participant(&self.pool, call_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::ConflictError("You're already in this call".to_string()));
        }

        // Add participant
        CallRepository::add_participant(
            &self.pool,
            call_id,
            user_id,
            call.call_type == CallType::Video,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Update call status to in_progress if it was ringing
        if call.status == CallStatus::Ringing {
            CallRepository::update_status(&self.pool, call_id, CallStatus::InProgress)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        self.get_call_response(call_id).await
    }

    pub async fn leave_call(&self, call_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        let call = CallRepository::find_by_id(&self.pool, call_id)
            .await
            .map_err(|_| AppError::NotFoundError("Call not found".to_string()))?;

        // Check if user is a participant
        if !CallRepository::is_participant(&self.pool, call_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::BadRequest("You're not in this call".to_string()));
        }

        // Remove participant
        CallRepository::remove_participant(&self.pool, call_id, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Check if there are any participants left
        let participant_count = CallRepository::get_participant_count(&self.pool, call_id)
            .await
            .unwrap_or(0);

        // End call if no participants left
        if participant_count == 0 {
            CallRepository::update_status(&self.pool, call_id, CallStatus::Ended)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn end_call(&self, call_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        let call = CallRepository::find_by_id(&self.pool, call_id)
            .await
            .map_err(|_| AppError::NotFoundError("Call not found".to_string()))?;

        // Only initiator can end the call
        if call.initiator_id != *user_id {
            return Err(AppError::AuthorizationError(
                "Only the call initiator can end the call".to_string(),
            ));
        }

        CallRepository::update_status(&self.pool, call_id, CallStatus::Ended)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn update_participant(
        &self,
        call_id: &Uuid,
        user_id: &Uuid,
        is_muted: Option<bool>,
        is_video_enabled: Option<bool>,
    ) -> Result<CallParticipantResponse, AppError> {
        // Check if user is a participant
        if !CallRepository::is_participant(&self.pool, call_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::BadRequest("You're not in this call".to_string()));
        }

        let participant = CallRepository::update_participant(
            &self.pool,
            call_id,
            user_id,
            is_muted,
            is_video_enabled,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let user = UserRepository::find_by_id(&self.pool, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(CallParticipantResponse {
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
            joined_at: participant.joined_at,
            is_muted: participant.is_muted,
            is_video_enabled: participant.is_video_enabled,
        })
    }

    pub async fn get_active_call(&self, channel_id: &Uuid) -> Result<Option<CallResponse>, AppError> {
        let call = CallRepository::find_active_by_channel(&self.pool, channel_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        match call {
            Some(c) => Ok(Some(self.get_call_response(&c.id).await?)),
            None => Ok(None),
        }
    }

    async fn get_call_response(&self, call_id: &Uuid) -> Result<CallResponse, AppError> {
        let call = CallRepository::find_by_id(&self.pool, call_id)
            .await
            .map_err(|_| AppError::NotFoundError("Call not found".to_string()))?;

        let initiator = UserRepository::find_by_id(&self.pool, &call.initiator_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let participants = CallRepository::find_participants(&self.pool, call_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut participant_responses = Vec::new();
        for participant in participants {
            let user = UserRepository::find_by_id(&self.pool, &participant.user_id)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            participant_responses.push(CallParticipantResponse {
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
                joined_at: participant.joined_at,
                is_muted: participant.is_muted,
                is_video_enabled: participant.is_video_enabled,
            });
        }

        Ok(CallResponse {
            id: call.id,
            channel_id: call.channel_id,
            initiator: UserResponse {
                id: initiator.id,
                email: initiator.email,
                username: initiator.username,
                display_name: initiator.display_name,
                avatar_url: initiator.avatar_url,
                status: initiator.status,
                status_message: initiator.status_message,
                last_seen: initiator.last_seen,
                created_at: initiator.created_at,
            },
            call_type: call.call_type,
            status: call.status,
            participants: participant_responses,
            started_at: call.started_at,
            ended_at: call.ended_at,
        })
    }
}
