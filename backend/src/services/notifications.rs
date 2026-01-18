//! Notification service

use shared::dto::NotificationResponse;
use shared::error::AppError;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::NotificationRepository;

pub struct NotificationService {
    pool: Arc<PgPool>,
}

impl NotificationService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_notification(
        &self,
        user_id: &Uuid,
        title: &str,
        body: &str,
        notification_type: &str,
        reference_id: Option<&Uuid>,
    ) -> Result<NotificationResponse, AppError> {
        let notification = NotificationRepository::create(
            &self.pool,
            user_id,
            title,
            body,
            notification_type,
            reference_id,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(NotificationResponse {
            id: notification.id,
            title: notification.title,
            body: notification.body,
            notification_type: notification.notification_type.to_string(),
            reference_id: notification.reference_id.and_then(|s| Uuid::parse_str(&s).ok()),
            read: notification.read,
            created_at: notification.created_at,
        })
    }

    pub async fn list_notifications(
        &self,
        user_id: &Uuid,
        unread_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<NotificationResponse>, AppError> {
        let notifications = NotificationRepository::find_by_user(
            &self.pool,
            user_id,
            unread_only,
            limit,
            offset,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(notifications
            .into_iter()
            .map(|n| NotificationResponse {
                id: n.id,
                title: n.title,
                body: n.body,
                notification_type: n.notification_type.to_string(),
                reference_id: n.reference_id.and_then(|s| Uuid::parse_str(&s).ok()),
                read: n.read,
                created_at: n.created_at,
            })
            .collect())
    }

    pub async fn mark_as_read(&self, notification_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        // Verify notification belongs to user
        let notification = NotificationRepository::find_by_id(&self.pool, notification_id)
            .await
            .map_err(|_| AppError::NotFoundError("Notification not found".to_string()))?;

        if notification.user_id != *user_id {
            return Err(AppError::AuthorizationError(
                "You don't have access to this notification".to_string(),
            ));
        }

        NotificationRepository::mark_as_read(&self.pool, notification_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn mark_all_as_read(&self, user_id: &Uuid) -> Result<(), AppError> {
        NotificationRepository::mark_all_as_read(&self.pool, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn get_unread_count(&self, user_id: &Uuid) -> Result<i64, AppError> {
        NotificationRepository::get_unread_count(&self.pool, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    // Helper methods for creating specific notification types
    pub async fn notify_new_message(
        &self,
        user_id: &Uuid,
        sender_name: &str,
        channel_name: &str,
        message_id: &Uuid,
    ) -> Result<NotificationResponse, AppError> {
        self.create_notification(
            user_id,
            &format!("New message from {}", sender_name),
            &format!("You have a new message in {}", channel_name),
            "new_message",
            Some(message_id),
        )
        .await
    }

    pub async fn notify_mention(
        &self,
        user_id: &Uuid,
        sender_name: &str,
        channel_name: &str,
        message_id: &Uuid,
    ) -> Result<NotificationResponse, AppError> {
        self.create_notification(
            user_id,
            &format!("{} mentioned you", sender_name),
            &format!("You were mentioned in {}", channel_name),
            "mention",
            Some(message_id),
        )
        .await
    }

    pub async fn notify_team_invite(
        &self,
        user_id: &Uuid,
        team_name: &str,
        team_id: &Uuid,
    ) -> Result<NotificationResponse, AppError> {
        self.create_notification(
            user_id,
            "Team Invitation",
            &format!("You've been invited to join {}", team_name),
            "team_invite",
            Some(team_id),
        )
        .await
    }

    pub async fn notify_call_started(
        &self,
        user_id: &Uuid,
        caller_name: &str,
        channel_name: &str,
        call_id: &Uuid,
    ) -> Result<NotificationResponse, AppError> {
        self.create_notification(
            user_id,
            "Incoming Call",
            &format!("{} started a call in {}", caller_name, channel_name),
            "call_started",
            Some(call_id),
        )
        .await
    }
}
