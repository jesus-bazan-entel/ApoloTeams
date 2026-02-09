//! Meeting service

use chrono::{DateTime, Utc};
use shared::dto::{
    CreateMeetingRequest, MeetingParticipantResponse, MeetingResponse, UpdateMeetingRequest,
    UserResponse,
};
use shared::error::AppError;
use shared::models::{MeetingResponseStatus, MeetingStatus, RecurrenceType};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::{MeetingRepository, UserRepository};

pub struct MeetingService {
    pool: Arc<PgPool>,
}

impl MeetingService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_meeting(
        &self,
        organizer_id: &Uuid,
        request: CreateMeetingRequest,
    ) -> Result<MeetingResponse, AppError> {
        // Validate dates
        if request.end_time <= request.start_time {
            return Err(AppError::BadRequest(
                "End time must be after start time".to_string(),
            ));
        }

        let meeting = MeetingRepository::create(
            &self.pool,
            organizer_id,
            &request.title,
            request.description.as_deref(),
            request.start_time,
            request.end_time,
            request.timezone.as_deref(),
            request.is_online.unwrap_or(true),
            request.location.as_deref(),
            request.recurrence.unwrap_or(RecurrenceType::None),
            request.channel_id,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Add invited participants
        if let Some(participant_ids) = request.participant_ids {
            for user_id in participant_ids {
                if user_id != *organizer_id {
                    // Don't re-add organizer
                    MeetingRepository::add_participant(&self.pool, &meeting.id, &user_id, false)
                        .await
                        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
                }
            }
        }

        self.get_meeting_response(&meeting.id).await
    }

    pub async fn get_meeting(
        &self,
        meeting_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<MeetingResponse, AppError> {
        // Check if user is a participant
        if !MeetingRepository::is_participant(&self.pool, meeting_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this meeting".to_string(),
            ));
        }

        self.get_meeting_response(meeting_id).await
    }

    pub async fn get_user_meetings(&self, user_id: &Uuid) -> Result<Vec<MeetingResponse>, AppError> {
        let meetings = MeetingRepository::find_by_user(&self.pool, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut responses = Vec::new();
        for meeting in meetings {
            responses.push(self.get_meeting_response(&meeting.id).await?);
        }

        Ok(responses)
    }

    pub async fn get_meetings_in_range(
        &self,
        user_id: &Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<MeetingResponse>, AppError> {
        let meetings = MeetingRepository::find_by_date_range(&self.pool, user_id, start_date, end_date)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut responses = Vec::new();
        for meeting in meetings {
            responses.push(self.get_meeting_response(&meeting.id).await?);
        }

        Ok(responses)
    }

    pub async fn update_meeting(
        &self,
        meeting_id: &Uuid,
        user_id: &Uuid,
        request: UpdateMeetingRequest,
    ) -> Result<MeetingResponse, AppError> {
        // Check if user is the organizer
        if !MeetingRepository::is_organizer(&self.pool, meeting_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "Only the organizer can update the meeting".to_string(),
            ));
        }

        // Validate dates if both are provided
        if let (Some(start), Some(end)) = (&request.start_time, &request.end_time) {
            if end <= start {
                return Err(AppError::BadRequest(
                    "End time must be after start time".to_string(),
                ));
            }
        }

        MeetingRepository::update(
            &self.pool,
            meeting_id,
            request.title.as_deref(),
            request.description.as_deref(),
            request.start_time,
            request.end_time,
            request.timezone.as_deref(),
            request.is_online,
            request.location.as_deref(),
            request.recurrence,
            request.status,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        self.get_meeting_response(meeting_id).await
    }

    pub async fn cancel_meeting(
        &self,
        meeting_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<MeetingResponse, AppError> {
        // Check if user is the organizer
        if !MeetingRepository::is_organizer(&self.pool, meeting_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "Only the organizer can cancel the meeting".to_string(),
            ));
        }

        MeetingRepository::update_status(&self.pool, meeting_id, MeetingStatus::Cancelled)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        self.get_meeting_response(meeting_id).await
    }

    pub async fn delete_meeting(&self, meeting_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        // Check if user is the organizer
        if !MeetingRepository::is_organizer(&self.pool, meeting_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "Only the organizer can delete the meeting".to_string(),
            ));
        }

        MeetingRepository::delete(&self.pool, meeting_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn invite_participants(
        &self,
        meeting_id: &Uuid,
        user_id: &Uuid,
        participant_ids: Vec<Uuid>,
    ) -> Result<MeetingResponse, AppError> {
        // Check if user is the organizer
        if !MeetingRepository::is_organizer(&self.pool, meeting_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "Only the organizer can invite participants".to_string(),
            ));
        }

        for pid in participant_ids {
            MeetingRepository::add_participant(&self.pool, meeting_id, &pid, false)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        self.get_meeting_response(meeting_id).await
    }

    pub async fn respond_to_meeting(
        &self,
        meeting_id: &Uuid,
        user_id: &Uuid,
        response: MeetingResponseStatus,
    ) -> Result<MeetingResponse, AppError> {
        // Check if user is a participant
        if !MeetingRepository::is_participant(&self.pool, meeting_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You are not invited to this meeting".to_string(),
            ));
        }

        MeetingRepository::update_participant_response(&self.pool, meeting_id, user_id, response)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        self.get_meeting_response(meeting_id).await
    }

    pub async fn remove_participant(
        &self,
        meeting_id: &Uuid,
        user_id: &Uuid,
        participant_id: &Uuid,
    ) -> Result<(), AppError> {
        // Check if user is the organizer
        if !MeetingRepository::is_organizer(&self.pool, meeting_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "Only the organizer can remove participants".to_string(),
            ));
        }

        // Cannot remove the organizer
        if MeetingRepository::is_organizer(&self.pool, meeting_id, participant_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::BadRequest(
                "Cannot remove the organizer from the meeting".to_string(),
            ));
        }

        MeetingRepository::remove_participant(&self.pool, meeting_id, participant_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_meeting_response(&self, meeting_id: &Uuid) -> Result<MeetingResponse, AppError> {
        let meeting = MeetingRepository::find_by_id(&self.pool, meeting_id)
            .await
            .map_err(|_| AppError::NotFoundError("Meeting not found".to_string()))?;

        let organizer = UserRepository::find_by_id(&self.pool, &meeting.organizer_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let participants = MeetingRepository::find_participants(&self.pool, meeting_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut participant_responses = Vec::new();
        for participant in participants {
            let user = UserRepository::find_by_id(&self.pool, &participant.user_id)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            participant_responses.push(MeetingParticipantResponse {
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
                response_status: participant.response_status,
                is_organizer: participant.is_organizer,
                invited_at: participant.invited_at,
                responded_at: participant.responded_at,
            });
        }

        Ok(MeetingResponse {
            id: meeting.id,
            title: meeting.title,
            description: meeting.description,
            organizer: UserResponse {
                id: organizer.id,
                email: organizer.email,
                username: organizer.username,
                display_name: organizer.display_name,
                avatar_url: organizer.avatar_url,
                status: organizer.status,
                status_message: organizer.status_message,
                last_seen: organizer.last_seen,
                created_at: organizer.created_at,
            },
            start_time: meeting.start_time,
            end_time: meeting.end_time,
            timezone: meeting.timezone,
            status: meeting.status,
            is_online: meeting.is_online,
            meeting_link: meeting.meeting_link,
            location: meeting.location,
            recurrence: meeting.recurrence,
            channel_id: meeting.channel_id,
            participants: participant_responses,
            created_at: meeting.created_at,
        })
    }
}
