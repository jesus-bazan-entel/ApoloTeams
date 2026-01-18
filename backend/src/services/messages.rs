//! Message service

use chrono::{DateTime, Utc};
use shared::dto::{
    FileAttachmentResponse, MessageResponse, ReactionResponse, SendMessageRequest,
    UpdateMessageRequest, UserResponse,
};
use shared::error::AppError;
use shared::models::MessageType;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::{ChannelRepository, FileRepository, MessageRepository, UserRepository};

pub struct MessageService {
    pool: Arc<PgPool>,
}

impl MessageService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn send_message(
        &self,
        channel_id: &Uuid,
        sender_id: &Uuid,
        request: SendMessageRequest,
    ) -> Result<MessageResponse, AppError> {
        // Verify sender has access to channel
        if !ChannelRepository::is_member(&self.pool, channel_id, sender_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this channel".to_string(),
            ));
        }

        let message_type = request.message_type.unwrap_or(MessageType::Text);

        let message = MessageRepository::create(
            &self.pool,
            channel_id,
            sender_id,
            &request.content,
            message_type,
            request.reply_to_id.as_ref(),
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Attach files if any
        if let Some(attachment_ids) = request.attachment_ids {
            for file_id in attachment_ids {
                let _ = FileRepository::attach_to_message(&self.pool, &file_id, &message.id).await;
            }
        }

        self.get_message_response(&message.id, sender_id).await
    }

    pub async fn get_message(
        &self,
        message_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<MessageResponse, AppError> {
        let message = MessageRepository::find_by_id(&self.pool, message_id)
            .await
            .map_err(|_| AppError::NotFoundError("Message not found".to_string()))?;

        // Verify user has access to channel
        if !ChannelRepository::is_member(&self.pool, &message.channel_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this message".to_string(),
            ));
        }

        self.get_message_response(message_id, user_id).await
    }

    pub async fn list_messages(
        &self,
        channel_id: &Uuid,
        user_id: &Uuid,
        limit: i64,
        before: Option<DateTime<Utc>>,
        after: Option<DateTime<Utc>>,
    ) -> Result<Vec<MessageResponse>, AppError> {
        // Verify user has access to channel
        if !ChannelRepository::is_member(&self.pool, channel_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this channel".to_string(),
            ));
        }

        let messages = MessageRepository::find_by_channel(&self.pool, channel_id, limit, before, after)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut responses = Vec::new();
        for message in messages {
            if let Ok(response) = self.get_message_response(&message.id, user_id).await {
                responses.push(response);
            }
        }

        // Reverse to get chronological order
        responses.reverse();

        Ok(responses)
    }

    pub async fn update_message(
        &self,
        message_id: &Uuid,
        user_id: &Uuid,
        request: UpdateMessageRequest,
    ) -> Result<MessageResponse, AppError> {
        let message = MessageRepository::find_by_id(&self.pool, message_id)
            .await
            .map_err(|_| AppError::NotFoundError("Message not found".to_string()))?;

        // Only sender can edit
        if message.sender_id != *user_id {
            return Err(AppError::AuthorizationError(
                "You can only edit your own messages".to_string(),
            ));
        }

        MessageRepository::update(&self.pool, message_id, &request.content)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        self.get_message_response(message_id, user_id).await
    }

    pub async fn delete_message(&self, message_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        let message = MessageRepository::find_by_id(&self.pool, message_id)
            .await
            .map_err(|_| AppError::NotFoundError("Message not found".to_string()))?;

        // Only sender can delete
        if message.sender_id != *user_id {
            return Err(AppError::AuthorizationError(
                "You can only delete your own messages".to_string(),
            ));
        }

        MessageRepository::delete(&self.pool, message_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn add_reaction(
        &self,
        message_id: &Uuid,
        user_id: &Uuid,
        emoji: &str,
    ) -> Result<(), AppError> {
        let message = MessageRepository::find_by_id(&self.pool, message_id)
            .await
            .map_err(|_| AppError::NotFoundError("Message not found".to_string()))?;

        // Verify user has access to channel
        if !ChannelRepository::is_member(&self.pool, &message.channel_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this message".to_string(),
            ));
        }

        // Check if already reacted
        if MessageRepository::has_user_reacted(&self.pool, message_id, user_id, emoji)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::ConflictError("You already reacted with this emoji".to_string()));
        }

        MessageRepository::add_reaction(&self.pool, message_id, user_id, emoji)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn remove_reaction(
        &self,
        message_id: &Uuid,
        user_id: &Uuid,
        emoji: &str,
    ) -> Result<(), AppError> {
        MessageRepository::remove_reaction(&self.pool, message_id, user_id, emoji)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn search_messages(
        &self,
        user_id: &Uuid,
        query: &str,
        channel_id: Option<&Uuid>,
        from_user_id: Option<&Uuid>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<MessageResponse>, i64), AppError> {
        // If channel_id is specified, verify access
        if let Some(cid) = channel_id {
            if !ChannelRepository::is_member(&self.pool, cid, user_id)
                .await
                .unwrap_or(false)
            {
                return Err(AppError::AuthorizationError(
                    "You don't have access to this channel".to_string(),
                ));
            }
        }

        let (messages, total_count) = MessageRepository::search(
            &self.pool,
            query,
            channel_id,
            from_user_id,
            from_date,
            to_date,
            limit,
            offset,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut responses = Vec::new();
        for message in messages {
            // Filter out messages from channels user doesn't have access to
            if ChannelRepository::is_member(&self.pool, &message.channel_id, user_id)
                .await
                .unwrap_or(false)
            {
                if let Ok(response) = self.get_message_response(&message.id, user_id).await {
                    responses.push(response);
                }
            }
        }

        Ok((responses, total_count))
    }

    async fn get_message_response(
        &self,
        message_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<MessageResponse, AppError> {
        let message = MessageRepository::find_by_id(&self.pool, message_id)
            .await
            .map_err(|_| AppError::NotFoundError("Message not found".to_string()))?;

        let sender = UserRepository::find_by_id(&self.pool, &message.sender_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get reply_to message if exists
        let reply_to = if let Some(reply_id) = message.reply_to_id {
            if let Ok(reply_msg) = MessageRepository::find_by_id(&self.pool, &reply_id).await {
                if let Ok(reply_sender) = UserRepository::find_by_id(&self.pool, &reply_msg.sender_id).await {
                    Some(Box::new(MessageResponse {
                        id: reply_msg.id,
                        channel_id: reply_msg.channel_id,
                        sender: UserResponse {
                            id: reply_sender.id,
                            email: reply_sender.email,
                            username: reply_sender.username,
                            display_name: reply_sender.display_name,
                            avatar_url: reply_sender.avatar_url,
                            status: reply_sender.status,
                            status_message: reply_sender.status_message,
                            last_seen: reply_sender.last_seen,
                            created_at: reply_sender.created_at,
                        },
                        content: reply_msg.content,
                        message_type: reply_msg.message_type,
                        reply_to: None,
                        reactions: vec![],
                        attachments: vec![],
                        edited: reply_msg.edited,
                        created_at: reply_msg.created_at,
                        updated_at: reply_msg.updated_at,
                    }))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Get reactions
        let reactions = MessageRepository::get_reactions(&self.pool, message_id)
            .await
            .unwrap_or_default();

        // Group reactions by emoji
        let mut reaction_map: HashMap<String, (i64, Vec<Uuid>)> = HashMap::new();
        for reaction in reactions {
            let entry = reaction_map.entry(reaction.emoji.clone()).or_insert((0, vec![]));
            entry.0 += 1;
            entry.1.push(reaction.user_id);
        }

        let reaction_responses: Vec<ReactionResponse> = reaction_map
            .into_iter()
            .map(|(emoji, (count, users))| ReactionResponse {
                emoji,
                count,
                reacted_by_me: users.contains(user_id),
                users,
            })
            .collect();

        // Get attachments
        let attachments = FileRepository::find_by_message(&self.pool, message_id)
            .await
            .unwrap_or_default();

        let attachment_responses: Vec<FileAttachmentResponse> = attachments
            .into_iter()
            .map(|f| FileAttachmentResponse {
                id: f.id,
                filename: f.filename,
                file_size: f.file_size,
                mime_type: f.mime_type,
                download_url: format!("/api/v1/files/{}/download", f.id),
                created_at: f.created_at,
            })
            .collect();

        Ok(MessageResponse {
            id: message.id,
            channel_id: message.channel_id,
            sender: UserResponse {
                id: sender.id,
                email: sender.email,
                username: sender.username,
                display_name: sender.display_name,
                avatar_url: sender.avatar_url,
                status: sender.status,
                status_message: sender.status_message,
                last_seen: sender.last_seen,
                created_at: sender.created_at,
            },
            content: message.content,
            message_type: message.message_type,
            reply_to,
            reactions: reaction_responses,
            attachments: attachment_responses,
            edited: message.edited,
            created_at: message.created_at,
            updated_at: message.updated_at,
        })
    }
}
