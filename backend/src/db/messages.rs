//! Message database operations

use chrono::{DateTime, Utc};
use shared::models::{Message, MessageType, Reaction};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct MessageRow {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub message_type: String,
    pub reply_to_id: Option<Uuid>,
    pub edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MessageRow> for Message {
    fn from(row: MessageRow) -> Self {
        Message {
            id: row.id,
            channel_id: row.channel_id,
            sender_id: row.sender_id,
            content: row.content,
            message_type: serde_json::from_str(&format!("\"{}\"", row.message_type)).unwrap_or_default(),
            reply_to_id: row.reply_to_id,
            edited: row.edited,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct ReactionRow {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

impl From<ReactionRow> for Reaction {
    fn from(row: ReactionRow) -> Self {
        Reaction {
            id: row.id,
            message_id: row.message_id,
            user_id: row.user_id,
            emoji: row.emoji,
            created_at: row.created_at,
        }
    }
}

pub struct MessageRepository;

impl MessageRepository {
    pub async fn create(
        pool: &PgPool,
        channel_id: &Uuid,
        sender_id: &Uuid,
        content: &str,
        message_type: MessageType,
        reply_to_id: Option<&Uuid>,
    ) -> Result<Message, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let type_str = serde_json::to_string(&message_type).unwrap().trim_matches('"').to_string();

        sqlx::query(
            r#"
            INSERT INTO messages (id, channel_id, sender_id, content, message_type, reply_to_id, edited, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&id)
        .bind(channel_id)
        .bind(sender_id)
        .bind(content)
        .bind(&type_str)
        .bind(reply_to_id)
        .bind(false)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &id).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Message, sqlx::Error> {
        let row: MessageRow = sqlx::query_as(
            r#"SELECT id, channel_id, sender_id, content, message_type, reply_to_id, edited, created_at, updated_at FROM messages WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_channel(
        pool: &PgPool,
        channel_id: &Uuid,
        limit: i64,
        before: Option<DateTime<Utc>>,
        after: Option<DateTime<Utc>>,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let rows: Vec<MessageRow> = match (before, after) {
            (Some(b), None) => {
                sqlx::query_as(
                    r#"SELECT id, channel_id, sender_id, content, message_type, reply_to_id, edited, created_at, updated_at 
                    FROM messages WHERE channel_id = $1 AND created_at < $2 
                    ORDER BY created_at DESC LIMIT $3"#,
                )
                .bind(channel_id)
                .bind(b)
                .bind(limit)
                .fetch_all(pool)
                .await?
            }
            (None, Some(a)) => {
                sqlx::query_as(
                    r#"SELECT id, channel_id, sender_id, content, message_type, reply_to_id, edited, created_at, updated_at 
                    FROM messages WHERE channel_id = $1 AND created_at > $2 
                    ORDER BY created_at DESC LIMIT $3"#,
                )
                .bind(channel_id)
                .bind(a)
                .bind(limit)
                .fetch_all(pool)
                .await?
            }
            (Some(b), Some(a)) => {
                sqlx::query_as(
                    r#"SELECT id, channel_id, sender_id, content, message_type, reply_to_id, edited, created_at, updated_at 
                    FROM messages WHERE channel_id = $1 AND created_at < $2 AND created_at > $3 
                    ORDER BY created_at DESC LIMIT $4"#,
                )
                .bind(channel_id)
                .bind(b)
                .bind(a)
                .bind(limit)
                .fetch_all(pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as(
                    r#"SELECT id, channel_id, sender_id, content, message_type, reply_to_id, edited, created_at, updated_at 
                    FROM messages WHERE channel_id = $1 
                    ORDER BY created_at DESC LIMIT $2"#,
                )
                .bind(channel_id)
                .bind(limit)
                .fetch_all(pool)
                .await?
            }
        };

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update(
        pool: &PgPool,
        id: &Uuid,
        content: &str,
    ) -> Result<Message, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"UPDATE messages SET content = $1, edited = $2, updated_at = $3 WHERE id = $4"#,
        )
        .bind(content)
        .bind(true)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, id).await
    }

    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM messages WHERE id = $1"#)
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn search(
        pool: &PgPool,
        query: &str,
        channel_id: Option<&Uuid>,
        from_user_id: Option<&Uuid>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Message>, i64), sqlx::Error> {
        let search_pattern = format!("%{}%", query);
        
        // Build query dynamically based on filters
        let base_query = "SELECT id, channel_id, sender_id, content, message_type, reply_to_id, edited, created_at, updated_at FROM messages WHERE content ILIKE $1";
        let count_base = "SELECT COUNT(*) FROM messages WHERE content ILIKE $1";
        
        // For simplicity, we'll use a basic query without dynamic filters for now
        let rows: Vec<MessageRow> = sqlx::query_as(
            &format!("{} ORDER BY created_at DESC LIMIT $2 OFFSET $3", base_query),
        )
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let (total_count,): (i64,) = sqlx::query_as(count_base)
            .bind(&search_pattern)
            .fetch_one(pool)
            .await?;

        Ok((rows.into_iter().map(|r| r.into()).collect(), total_count))
    }

    pub async fn get_last_message(
        pool: &PgPool,
        channel_id: &Uuid,
    ) -> Result<Option<Message>, sqlx::Error> {
        let row: Option<MessageRow> = sqlx::query_as(
            r#"SELECT id, channel_id, sender_id, content, message_type, reply_to_id, edited, created_at, updated_at 
            FROM messages WHERE channel_id = $1 ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(channel_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    // Reaction operations
    pub async fn add_reaction(
        pool: &PgPool,
        message_id: &Uuid,
        user_id: &Uuid,
        emoji: &str,
    ) -> Result<Reaction, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO reactions (id, message_id, user_id, emoji, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&id)
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .bind(&now)
        .execute(pool)
        .await?;

        let row: ReactionRow = sqlx::query_as(
            r#"SELECT id, message_id, user_id, emoji, created_at FROM reactions WHERE id = $1"#,
        )
        .bind(&id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn remove_reaction(
        pool: &PgPool,
        message_id: &Uuid,
        user_id: &Uuid,
        emoji: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"DELETE FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3"#,
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_reactions(
        pool: &PgPool,
        message_id: &Uuid,
    ) -> Result<Vec<Reaction>, sqlx::Error> {
        let rows: Vec<ReactionRow> = sqlx::query_as(
            r#"SELECT id, message_id, user_id, emoji, created_at FROM reactions WHERE message_id = $1 ORDER BY created_at"#,
        )
        .bind(message_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn has_user_reacted(
        pool: &PgPool,
        message_id: &Uuid,
        user_id: &Uuid,
        emoji: &str,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3"#,
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }
}
