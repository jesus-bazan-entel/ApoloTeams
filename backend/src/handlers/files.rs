//! File handlers

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::get_user_id_from_request;
use crate::services::Services;

pub async fn upload_file(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    mut payload: Multipart,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;

    let mut channel_id: Option<Uuid> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut file_data: Vec<u8> = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            crate::error::ApiError(shared::error::AppError::FileUploadError(e.to_string()))
        })?;

        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap_or("");

        match field_name {
            "channel_id" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = chunk.map_err(|e| {
                        crate::error::ApiError(shared::error::AppError::FileUploadError(e.to_string()))
                    })?;
                    data.extend_from_slice(&chunk);
                }
                let id_str = String::from_utf8(data).map_err(|e| {
                    crate::error::ApiError(shared::error::AppError::FileUploadError(e.to_string()))
                })?;
                channel_id = Some(Uuid::parse_str(&id_str).map_err(|e| {
                    crate::error::ApiError(shared::error::AppError::FileUploadError(e.to_string()))
                })?);
            }
            "file" => {
                filename = content_disposition.get_filename().map(|s| s.to_string());
                content_type = field.content_type().map(|m| m.to_string());

                while let Some(chunk) = field.next().await {
                    let chunk = chunk.map_err(|e| {
                        crate::error::ApiError(shared::error::AppError::FileUploadError(e.to_string()))
                    })?;
                    file_data.extend_from_slice(&chunk);
                }
            }
            _ => {}
        }
    }

    let channel_id = channel_id.ok_or_else(|| {
        crate::error::ApiError(shared::error::AppError::BadRequest(
            "channel_id is required".to_string(),
        ))
    })?;

    let filename = filename.ok_or_else(|| {
        crate::error::ApiError(shared::error::AppError::BadRequest(
            "filename is required".to_string(),
        ))
    })?;

    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

    let response = services
        .files
        .upload_file(&channel_id, &user_id, &filename, &content_type, &file_data)
        .await?;

    Ok(HttpResponse::Created().json(response))
}

pub async fn get_file(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let file_id = path.into_inner();

    let file = services.files.get_file(&file_id, &user_id).await?;
    Ok(HttpResponse::Ok().json(file))
}

pub async fn download_file(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let file_id = path.into_inner();

    let (file_path, filename, mime_type) = services.files.get_file_path(&file_id, &user_id).await?;

    let file_content = tokio::fs::read(&file_path).await.map_err(|e| {
        crate::error::ApiError(shared::error::AppError::InternalError(e.to_string()))
    })?;

    Ok(HttpResponse::Ok()
        .content_type(mime_type)
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(file_content))
}

pub async fn delete_file(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let file_id = path.into_inner();

    services.files.delete_file(&file_id, &user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
