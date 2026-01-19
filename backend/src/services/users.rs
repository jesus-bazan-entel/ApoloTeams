//! User service

use shared::dto::{UpdateUserRequest, UserResponse};
use shared::error::AppError;
use shared::models::UserStatus;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::UserRepository;

pub struct UserService {
    pool: Arc<PgPool>,
}

impl UserService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn get_user(&self, user_id: &Uuid) -> Result<UserResponse, AppError> {
        let user = UserRepository::find_by_id(&self.pool, user_id)
            .await
            .map_err(|_| AppError::NotFoundError("User not found".to_string()))?;

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            status: user.status,
            status_message: user.status_message,
            last_seen: user.last_seen,
            created_at: user.created_at,
        })
    }

    pub async fn update_user(
        &self,
        user_id: &Uuid,
        request: UpdateUserRequest,
    ) -> Result<UserResponse, AppError> {
        let user = UserRepository::update(
            &self.pool,
            user_id,
            request.display_name.as_deref(),
            request.avatar_url.as_deref(),
            request.status,
            request.status_message.as_deref(),
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            status: user.status,
            status_message: user.status_message,
            last_seen: user.last_seen,
            created_at: user.created_at,
        })
    }

    pub async fn update_status(
        &self,
        user_id: &Uuid,
        status: UserStatus,
        status_message: Option<&str>,
    ) -> Result<UserResponse, AppError> {
        let user = UserRepository::update(
            &self.pool,
            user_id,
            None,
            None,
            Some(status),
            status_message,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            status: user.status,
            status_message: user.status_message,
            last_seen: user.last_seen,
            created_at: user.created_at,
        })
    }

    pub async fn change_password(
        &self,
        user_id: &Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
            Argon2,
        };

        let user = UserRepository::find_by_id(&self.pool, user_id)
            .await
            .map_err(|_| AppError::NotFoundError("User not found".to_string()))?;

        // Verify current password
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if Argon2::default()
            .verify_password(current_password.as_bytes(), &parsed_hash)
            .is_err()
        {
            return Err(AppError::AuthenticationError("Current password is incorrect".to_string()));
        }

        // Hash new password
        let salt = SaltString::generate(&mut OsRng);
        let new_hash = Argon2::default()
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .to_string();

        UserRepository::update_password(&self.pool, user_id, &new_hash)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn search_users(
        &self,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserResponse>, AppError> {
        let users = UserRepository::search(&self.pool, query, limit, offset)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(users
            .into_iter()
            .map(|user| UserResponse {
                id: user.id,
                email: user.email,
                username: user.username,
                display_name: user.display_name,
                avatar_url: user.avatar_url,
                status: user.status,
                status_message: user.status_message,
                last_seen: user.last_seen,
                created_at: user.created_at,
            })
            .collect())
    }

    pub async fn update_last_seen(&self, user_id: &Uuid) -> Result<(), AppError> {
        UserRepository::update_last_seen(&self.pool, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}
