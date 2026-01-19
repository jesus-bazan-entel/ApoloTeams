//! File service

use shared::dto::{FileAttachmentResponse, UploadFileResponse};
use shared::error::AppError;
use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::{ChannelRepository, FileRepository};

pub struct FileService {
    pool: Arc<PgPool>,
    config: Arc<AppConfig>,
}

impl FileService {
    pub fn new(pool: Arc<PgPool>, config: Arc<AppConfig>) -> Self {
        Self { pool, config }
    }

    pub async fn upload_file(
        &self,
        channel_id: &Uuid,
        uploader_id: &Uuid,
        filename: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<UploadFileResponse, AppError> {
        // Verify user has access to channel
        if !ChannelRepository::is_member(&self.pool, channel_id, uploader_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this channel".to_string(),
            ));
        }

        // Check file size
        if data.len() > self.config.storage.max_file_size {
            return Err(AppError::FileUploadError(format!(
                "File size exceeds maximum allowed size of {} bytes",
                self.config.storage.max_file_size
            )));
        }

        // Generate unique filename
        let file_id = Uuid::new_v4();
        let extension = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let storage_filename = if extension.is_empty() {
            file_id.to_string()
        } else {
            format!("{}.{}", file_id, extension)
        };

        // Create channel directory if it doesn't exist
        let channel_dir = Path::new(&self.config.storage.upload_path).join(channel_id.to_string());
        fs::create_dir_all(&channel_dir)
            .await
            .map_err(|e| AppError::FileUploadError(e.to_string()))?;

        // Write file
        let file_path = channel_dir.join(&storage_filename);
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| AppError::FileUploadError(e.to_string()))?;

        file.write_all(data)
            .await
            .map_err(|e| AppError::FileUploadError(e.to_string()))?;

        // Store in database
        let storage_path = format!("{}/{}", channel_id, storage_filename);
        let attachment = FileRepository::create(
            &self.pool,
            channel_id,
            uploader_id,
            filename,
            data.len() as i64,
            content_type,
            &storage_path,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(UploadFileResponse {
            id: attachment.id,
            filename: attachment.filename,
            file_size: attachment.file_size,
            mime_type: attachment.mime_type,
        })
    }

    pub async fn get_file(
        &self,
        file_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<FileAttachmentResponse, AppError> {
        let file = FileRepository::find_by_id(&self.pool, file_id)
            .await
            .map_err(|_| AppError::NotFoundError("File not found".to_string()))?;

        // Verify user has access to channel
        if !ChannelRepository::is_member(&self.pool, &file.channel_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this file".to_string(),
            ));
        }

        Ok(FileAttachmentResponse {
            id: file.id,
            filename: file.filename,
            file_size: file.file_size,
            mime_type: file.mime_type,
            download_url: format!("/api/v1/files/{}/download", file.id),
            created_at: file.created_at,
        })
    }

    pub async fn get_file_path(
        &self,
        file_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(String, String, String), AppError> {
        let file = FileRepository::find_by_id(&self.pool, file_id)
            .await
            .map_err(|_| AppError::NotFoundError("File not found".to_string()))?;

        // Verify user has access to channel
        if !ChannelRepository::is_member(&self.pool, &file.channel_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this file".to_string(),
            ));
        }

        let full_path = Path::new(&self.config.storage.upload_path)
            .join(&file.storage_path)
            .to_string_lossy()
            .to_string();

        Ok((full_path, file.filename, file.mime_type))
    }

    pub async fn delete_file(&self, file_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        let file = FileRepository::find_by_id(&self.pool, file_id)
            .await
            .map_err(|_| AppError::NotFoundError("File not found".to_string()))?;

        // Only uploader can delete
        if file.uploader_id != *user_id {
            return Err(AppError::AuthorizationError(
                "You can only delete your own files".to_string(),
            ));
        }

        // Delete from filesystem
        let full_path = Path::new(&self.config.storage.upload_path).join(&file.storage_path);
        if full_path.exists() {
            fs::remove_file(&full_path)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }

        // Delete from database
        FileRepository::delete(&self.pool, file_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn list_channel_files(
        &self,
        channel_id: &Uuid,
        user_id: &Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<FileAttachmentResponse>, AppError> {
        // Verify user has access to channel
        if !ChannelRepository::is_member(&self.pool, channel_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::AuthorizationError(
                "You don't have access to this channel".to_string(),
            ));
        }

        let files = FileRepository::find_by_channel(&self.pool, channel_id, limit, offset)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(files
            .into_iter()
            .map(|f| FileAttachmentResponse {
                id: f.id,
                filename: f.filename,
                file_size: f.file_size,
                mime_type: f.mime_type,
                download_url: format!("/api/v1/files/{}/download", f.id),
                created_at: f.created_at,
            })
            .collect())
    }
}
