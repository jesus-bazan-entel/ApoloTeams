//! File attachment database operations

use chrono::{DateTime, Utc};
use shared::models::FileAttachment;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct FileAttachmentRow {
    pub id: String,
    pub message_id: Option<String>,
    pub channel_id: String,
    pub uploader_id: String,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: String,
    pub storage_path: String,
    pub created_at: String,
}

impl From<FileAttachmentRow> for FileAttachment {
    fn from(row: FileAttachmentRow) -> Self {
        FileAttachment {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            message_id: row.message_id.and_then(|s| Uuid::parse_str(&s).ok()),
            channel_id: Uuid::parse_str(&row.channel_id).unwrap_or_default(),
            uploader_id: Uuid::parse_str(&row.uploader_id).unwrap_or_default(),
            filename: row.filename,
            file_size: row.file_size,
            mime_type: row.mime_type,
            storage_path: row.storage_path,
            created_at: DateTime::parse_from_rfc3339(&row.created_at).unwrap().with_timezone(&Utc),
        }
    }
}

pub struct FileRepository;

impl FileRepository {
    pub async fn create(
        pool: &SqlitePool,
        channel_id: &Uuid,
        uploader_id: &Uuid,
        filename: &str,
        file_size: i64,
        mime_type: &str,
        storage_path: &str,
    ) -> Result<FileAttachment, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO file_attachments (id, channel_id, uploader_id, filename, file_size, mime_type, storage_path, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(channel_id.to_string())
        .bind(uploader_id.to_string())
        .bind(filename)
        .bind(file_size)
        .bind(mime_type)
        .bind(storage_path)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &Uuid::parse_str(&id).unwrap()).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &Uuid) -> Result<FileAttachment, sqlx::Error> {
        let row: FileAttachmentRow = sqlx::query_as(
            r#"SELECT * FROM file_attachments WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_message(pool: &SqlitePool, message_id: &Uuid) -> Result<Vec<FileAttachment>, sqlx::Error> {
        let rows: Vec<FileAttachmentRow> = sqlx::query_as(
            r#"SELECT * FROM file_attachments WHERE message_id = ? ORDER BY created_at"#,
        )
        .bind(message_id.to_string())
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn find_by_channel(
        pool: &SqlitePool,
        channel_id: &Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<FileAttachment>, sqlx::Error> {
        let rows: Vec<FileAttachmentRow> = sqlx::query_as(
            r#"SELECT * FROM file_attachments WHERE channel_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"#,
        )
        .bind(channel_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn attach_to_message(
        pool: &SqlitePool,
        file_id: &Uuid,
        message_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE file_attachments SET message_id = ? WHERE id = ?"#,
        )
        .bind(message_id.to_string())
        .bind(file_id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(pool: &SqlitePool, id: &Uuid) -> Result<FileAttachment, sqlx::Error> {
        let file = Self::find_by_id(pool, id).await?;

        sqlx::query(r#"DELETE FROM file_attachments WHERE id = ?"#)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(file)
    }

    pub async fn is_owner(
        pool: &SqlitePool,
        file_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM file_attachments WHERE id = ? AND uploader_id = ?"#,
        )
        .bind(file_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }
}
