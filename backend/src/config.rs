//! Application configuration

use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry: i64,  // seconds
    pub refresh_token_expiry: i64, // seconds
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub upload_path: String,
    pub max_file_size: usize, // bytes
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config = Self {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "sqlite:./data/rust_teams.db?mode=rwc&create=true".to_string()),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string()),
                access_token_expiry: env::var("JWT_ACCESS_TOKEN_EXPIRY")
                    .unwrap_or_else(|_| "3600".to_string()) // 1 hour
                    .parse()
                    .unwrap_or(3600),
                refresh_token_expiry: env::var("JWT_REFRESH_TOKEN_EXPIRY")
                    .unwrap_or_else(|_| "604800".to_string()) // 7 days
                    .parse()
                    .unwrap_or(604800),
            },
            storage: StorageConfig {
                upload_path: env::var("UPLOAD_PATH").unwrap_or_else(|_| "./uploads".to_string()),
                max_file_size: env::var("MAX_FILE_SIZE")
                    .unwrap_or_else(|_| "104857600".to_string()) // 100MB
                    .parse()
                    .unwrap_or(104857600),
            },
        };

        Ok(config)
    }
}
