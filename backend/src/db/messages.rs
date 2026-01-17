//! Message database operations

use chrono::{DateTime, Utc};
use shared::models::{Message, MessageType, Reaction};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct MessageRow {
    pub id: String,
    pub channel_id: String,
    pub sender_id: String,
    pub content: String,
    pub message_type: String,
    pub reply_to_id: Option<String>,
    pub edited: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<MessageRow> for Message {
    fn from(row: MessageRow) -> Self {
        Message {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            channel_id: Uuid::parse_str(&row.channel_id).unwrap_or_default(),
            sender_id: Uuid::parse_str(&row.sender_id).unwrap_or_default(),
            content: row.content,
            message_type: serde_json::from_str(&format!("\"{}\"", row.message_type)).unwrap_or_default(),
            reply_to_id: row.reply_to_id.and_then(|s| Uuid::parse_str(&s).ok()),
            edited: row.edited,
            created_at: DateTime::parse_from_rfc3339(&row.created_at).unwrap().with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at).unwrap().with_timezone(&Utc),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct ReactionRow {
    pub id: String,
    pub message_id: String,
    pub user_id: String,
    pub emoji: String,
    pub created_at: String,
}

impl From<ReactionRow> for Reaction {
    fn from(row: ReactionRow) -> Self {
        Reaction {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            message_id: Uuid::parse_str(&row.message_id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            emoji: row.emoji,
            created_at: DateTime::parse_from_rfc3339(&row.created_at).unwrap().with_timezone(&Utc),
        }
    }
}

pub struct MessageRepository;

impl MessageRepository {
    pub async fn create(
        pool: &SqlitePool,
        channel_id: &Uuid,
        sender_id: &Uuid,
        content: &str,
        message_type: MessageType,
        reply_to_id: Option<&Uuid>,
    ) -> Result<Message, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let type_str = serde_json::to_string(&message_type).unwrap().trim_matches('"').to_string();

        sqlx::query(
            r#"
            INSERT INTO messages (id, channel_id, sender_id, content, message_type, reply_to_id, edited, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(channel_id.to_string())
        .bind(sender_id.to_string())
        .bind(content)
        .bind(&type_str)
        .bind(reply_to_id.map(|r| r.to_string()))
        .bind(false)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &Uuid::parse_str(&id).unwrap()).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &Uuid) -> Result<Message, sqlx::Error> {
        let row: MessageRow = sqlx::query_as(
            r#"SELECT * FROM messages WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_channel(
        pool: &SqlitePool,
        channel_id: &Uuid,
        limit: i64,
        before: Option<DateTime<Utc>>,
        after: Option<DateTime<Utc>>,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM messages WHERE channel_id = ?");
        let mut params: Vec<String> = vec![channel_id.to_string()];

        if let Some(b) = before {
            query.push_str(" AND created_at < ?");
            params.push(b.to_rfc3339());
        }

        if let Some(a) = after {
            query.push_str(" AND created_at > ?");
            params.push(a.to_rfc3339());
        }

        query.push_str(" ORDER BY created_at DESC LIMIT ?");
        params.push(limit.to_string());

        let mut q = sqlx::query_as::<_, MessageRow>(&query);
        for param in &params {
            q = q.bind(param);
        }

        let rows: Vec<MessageRow> = q.fetch_all(pool).await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update(
        pool: &SqlitePool,
        id: &Uuid,
        content: &str,
    ) -> Result<Message, sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"UPDATE messages SET content = ?, edited = ?, updated_at = ? WHERE id = ?"#,
        )
        .bind(content)
        .bind(true)
        .bind(&now)
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Self::find_by_id(pool, id).await
    }

    pub async fn delete(pool: &SqlitePool, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM messages WHERE id = ?"#)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn search(
        pool: &SqlitePool,
        query: &str,
        channel_id: Option<&Uuid>,
        from_user_id: Option<&Uuid>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Message>, i64), sqlx::Error> {
        let search_pattern = format!("%{}%", query);
        let mut sql = String::from("SELECT * FROM messages WHERE content LIKE ?");
        let mut count_sql = String::from("SELECT COUNT(*) FROM messages WHERE content LIKE ?");
        let mut params: Vec<String> = vec![search_pattern.clone()];

        if let Some(cid) = channel_id {
            sql.push_str(" AND channel_id = ?");
            count_sql.push_str(" AND channel_id = ?");
            params.push(cid.to_string());
        }

        if let Some(uid) = from_user_id {
            sql.push_str(" AND sender_id = ?");
            count_sql.push_str(" AND sender_id = ?");
            params.push(uid.to_string());
        }

        if let Some(fd) = from_date {
            sql.push_str(" AND created_at >= ?");
            count_sql.push_str(" AND created_at >= ?");
            params.push(fd.to_rfc3339());
        }

        if let Some(td) = to_date {
            sql.push_str(" AND created_at <= ?");
            count_sql.push_str(" AND created_at <= ?");
            params.push(td.to_rfc3339());
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        // Get count
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for param in &params {
            count_q = count_q.bind(param);
        }
        let (total_count,) = count_q.fetch_one(pool).await?;

        // Get messages
        params.push(limit.to_string());
        params.push(offset.to_string());

        let mut q = sqlx::query_as::<_, MessageRow>(&sql);
        for param in &params {
            q = q.bind(param);
        }

        let rows: Vec<MessageRow> = q.fetch_all(pool).await?;
        Ok((rows.into_iter().map(|r| r.into()).collect(), total_count))
    }

    pub async fn get_last_message(
        pool: &SqlitePool,
        channel_id: &Uuid,
    ) -> Result<Option<Message>, sqlx::Error> {
        let row: Option<MessageRow> = sqlx::query_as(
            r#"SELECT * FROM messages WHERE channel_id = ? ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(channel_id.to_string())
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    // Reaction operations
    pub async fn add_reaction(
        pool: &SqlitePool,
        message_id: &Uuid,
        user_id: &Uuid,
        emoji: &str,
    ) -> Result<Reaction, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO reactions (id, message_id, user_id, emoji, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(message_id.to_string())
        .bind(user_id.to_string())
        .bind(emoji)
        .bind(&now)
        .execute(pool)
        .await?;

        let row: ReactionRow = sqlx::query_as(
            r#"SELECT * FROM reactions WHERE id = ?"#,
        )
        .bind(&id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn remove_reaction(
        pool: &SqlitePool,
        message_id: &Uuid,
        user_id: &Uuid,
        emoji: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"DELETE FROM reactions WHERE message_id = ? AND user_id = ? AND emoji = ?"#,
        )
        .bind(message_id.to_string())
        .bind(user_id.to_string())
        .bind(emoji)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_reactions(
        pool: &SqlitePool,
        message_id: &Uuid,
    ) -> Result<Vec<Reaction>, sqlx::Error> {
        let rows: Vec<ReactionRow> = sqlx::query_as(
            r#"SELECT * FROM reactions WHERE message_id = ? ORDER BY created_at"#,
        )
        .bind(message_id.to_string())
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn has_user_reacted(
        pool: &SqlitePool,
        message_id: &Uuid,
        user_id: &Uuid,
        emoji: &str,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM reactions WHERE message_id = ? AND user_id = ? AND emoji = ?"#,
        )
        .bind(message_id.to_string())
        .bind(user_id.to_string())
        .bind(emoji)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }
}
