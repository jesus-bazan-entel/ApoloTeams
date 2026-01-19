//! Authentication service

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use shared::dto::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use shared::error::AppError;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::UserRepository;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

pub struct AuthService {
    pool: Arc<PgPool>,
    config: Arc<AppConfig>,
}

impl AuthService {
    pub fn new(pool: Arc<PgPool>, config: Arc<AppConfig>) -> Self {
        Self { pool, config }
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse, AppError> {
        // Check if email already exists
        if UserRepository::exists_by_email(&self.pool, &request.email).await.map_err(|e| AppError::DatabaseError(e.to_string()))? {
            return Err(AppError::ConflictError("Email already registered".to_string()));
        }

        // Check if username already exists
        if UserRepository::exists_by_username(&self.pool, &request.username).await.map_err(|e| AppError::DatabaseError(e.to_string()))? {
            return Err(AppError::ConflictError("Username already taken".to_string()));
        }

        // Hash password
        let password_hash = self.hash_password(&request.password)?;

        // Create user
        let user = UserRepository::create(
            &self.pool,
            &request.email,
            &request.username,
            &request.display_name,
            &password_hash,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Generate tokens
        self.generate_auth_response(&user.id).await
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AppError> {
        // Find user by email
        let user = UserRepository::find_by_email(&self.pool, &request.email)
            .await
            .map_err(|_| AppError::AuthenticationError("Invalid email or password".to_string()))?;

        // Verify password
        if !self.verify_password(&request.password, &user.password_hash)? {
            return Err(AppError::AuthenticationError("Invalid email or password".to_string()));
        }

        // Update last seen
        let _ = UserRepository::update_last_seen(&self.pool, &user.id).await;

        // Generate tokens
        self.generate_auth_response(&user.id).await
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse, AppError> {
        // Verify refresh token
        let claims = self.verify_token(refresh_token)?;

        if claims.token_type != "refresh" {
            return Err(AppError::AuthenticationError("Invalid token type".to_string()));
        }

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::AuthenticationError("Invalid token".to_string()))?;

        // Verify user still exists
        let _ = UserRepository::find_by_id(&self.pool, &user_id)
            .await
            .map_err(|_| AppError::AuthenticationError("User not found".to_string()))?;

        // Generate new tokens
        self.generate_auth_response(&user_id).await
    }

    pub fn verify_access_token(&self, token: &str) -> Result<Uuid, AppError> {
        let claims = self.verify_token(token)?;

        if claims.token_type != "access" {
            return Err(AppError::AuthenticationError("Invalid token type".to_string()));
        }

        Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::AuthenticationError("Invalid token".to_string()))
    }

    fn hash_password(&self, password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AppError::InternalError(e.to_string()))
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    fn generate_access_token(&self, user_id: &Uuid) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.jwt.access_token_expiry);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type: "access".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt.secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalError(e.to_string()))
    }

    fn generate_refresh_token(&self, user_id: &Uuid) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.jwt.refresh_token_expiry);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type: "refresh".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt.secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalError(e.to_string()))
    }

    fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| AppError::AuthenticationError(e.to_string()))
    }

    async fn generate_auth_response(&self, user_id: &Uuid) -> Result<AuthResponse, AppError> {
        let user = UserRepository::find_by_id(&self.pool, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let access_token = self.generate_access_token(user_id)?;
        let refresh_token = self.generate_refresh_token(user_id)?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt.access_token_expiry,
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
        })
    }
}
