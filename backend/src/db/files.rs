//! File attachment database operations

use chrono::{DateTime, Utc};
use shared::models::FileAttachment;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct FileAttachmentRow {
    pub id: Uuid,
    pub message_id: Option<Uuid>,
    pub channel_id: Uuid,
    pub uploader_id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

impl From<FileAttachmentRow> for FileAttachment {
    fn from(row: FileAttachmentRow) -> Self {
        FileAttachment {
            id: row.id,
            message_id: row.message_id,
            channel_id: row.channel_id,
            uploader_id: row.uploader_id,
            filename: row.filename,
            file_size: row.file_size,
            mime_type: row.mime_type,
            storage_path: row.storage_path,
            created_at: row.created_at,
        }
    }
}

pub struct FileRepository;

impl FileRepository {
    pub async fn create(
        pool: &PgPool,
        channel_id: &Uuid,
        uploader_id: &Uuid,
        filename: &str,
        file_size: i64,
        mime_type: &str,
        storage_path: &str,
    ) -> Result<FileAttachment, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO file_attachments (id, channel_id, uploader_id, filename, file_size, mime_type, storage_path, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&id)
        .bind(channel_id)
        .bind(uploader_id)
        .bind(filename)
        .bind(file_size)
        .bind(mime_type)
        .bind(storage_path)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &id).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<FileAttachment, sqlx::Error> {
        let row: FileAttachmentRow = sqlx::query_as(
            r#"SELECT id, message_id, channel_id, uploader_id, filename, file_size, mime_type, storage_path, created_at FROM file_attachments WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_message(pool: &PgPool, message_id: &Uuid) -> Result<Vec<FileAttachment>, sqlx::Error> {
        let rows: Vec<FileAttachmentRow> = sqlx::query_as(
            r#"SELECT id, message_id, channel_id, uploader_id, filename, file_size, mime_type, storage_path, created_at FROM file_attachments WHERE message_id = $1 ORDER BY created_at"#,
        )
        .bind(message_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn find_by_channel(
        pool: &PgPool,
        channel_id: &Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<FileAttachment>, sqlx::Error> {
        let rows: Vec<FileAttachmentRow> = sqlx::query_as(
            r#"SELECT id, message_id, channel_id, uploader_id, filename, file_size, mime_type, storage_path, created_at FROM file_attachments WHERE channel_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(channel_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn attach_to_message(
        pool: &PgPool,
        file_id: &Uuid,
        message_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE file_attachments SET message_id = $1 WHERE id = $2"#,
        )
        .bind(message_id)
        .bind(file_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<FileAttachment, sqlx::Error> {
        let file = Self::find_by_id(pool, id).await?;

        sqlx::query(r#"DELETE FROM file_attachments WHERE id = $1"#)
            .bind(id)
            .execute(pool)
            .await?;

        Ok(file)
    }

    pub async fn is_owner(
        pool: &PgPool,
        file_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM file_attachments WHERE id = $1 AND uploader_id = $2"#,
        )
        .bind(file_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }
}
