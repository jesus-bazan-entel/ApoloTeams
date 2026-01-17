//! Rust Teams Backend Server
//! 
//! Main entry point for the Actix-web server.

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod handlers;
#[path = "middleware.rs"]
mod app_middleware;
mod services;
mod websocket;

use crate::config::AppConfig;
use crate::services::Services;
use crate::websocket::WebSocketServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::load().expect("Failed to load configuration");
    let server_addr = format!("{}:{}", config.server.host, config.server.port);

    info!("Starting Rust Teams server on {}", server_addr);

    // Create database pool
    let pool = SqlitePoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    info!("Database migrations completed");

    // Initialize services
    let services = Arc::new(Services::new(pool.clone(), config.clone()));

    // Initialize WebSocket server
    let ws_server = Arc::new(WebSocketServer::new());

    // Create upload directory if it doesn't exist
    tokio::fs::create_dir_all(&config.storage.upload_path)
        .await
        .expect("Failed to create upload directory");

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(web::Data::new(services.clone()))
            .app_data(web::Data::new(ws_server.clone()))
            .app_data(web::Data::new(config.clone()))
            // Health check
            .route("/health", web::get().to(handlers::health::health_check))
            // API routes
            .service(
                web::scope("/api/v1")
                    // Authentication
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(handlers::auth::register))
                            .route("/login", web::post().to(handlers::auth::login))
                            .route("/refresh", web::post().to(handlers::auth::refresh_token))
                            .route("/logout", web::post().to(handlers::auth::logout))
                    )
                    // Users
                    .service(
                        web::scope("/users")
                            .route("/me", web::get().to(handlers::users::get_current_user))
                            .route("/me", web::patch().to(handlers::users::update_current_user))
                            .route("/me/password", web::put().to(handlers::users::change_password))
                            .route("/{user_id}", web::get().to(handlers::users::get_user))
                            .route("/search", web::get().to(handlers::users::search_users))
                    )
                    // Teams
                    .service(
                        web::scope("/teams")
                            .route("", web::get().to(handlers::teams::list_teams))
                            .route("", web::post().to(handlers::teams::create_team))
                            .route("/{team_id}", web::get().to(handlers::teams::get_team))
                            .route("/{team_id}", web::patch().to(handlers::teams::update_team))
                            .route("/{team_id}", web::delete().to(handlers::teams::delete_team))
                            .route("/{team_id}/members", web::get().to(handlers::teams::list_team_members))
                            .route("/{team_id}/members", web::post().to(handlers::teams::add_team_member))
                            .route("/{team_id}/members/{user_id}", web::patch().to(handlers::teams::update_team_member))
                            .route("/{team_id}/members/{user_id}", web::delete().to(handlers::teams::remove_team_member))
                            .route("/{team_id}/channels", web::get().to(handlers::channels::list_team_channels))
                    )
                    // Channels
                    .service(
                        web::scope("/channels")
                            .route("", web::get().to(handlers::channels::list_channels))
                            .route("", web::post().to(handlers::channels::create_channel))
                            .route("/dm", web::post().to(handlers::channels::create_dm_channel))
                            .route("/{channel_id}", web::get().to(handlers::channels::get_channel))
                            .route("/{channel_id}", web::patch().to(handlers::channels::update_channel))
                            .route("/{channel_id}", web::delete().to(handlers::channels::delete_channel))
                            .route("/{channel_id}/members", web::get().to(handlers::channels::list_channel_members))
                            .route("/{channel_id}/members", web::post().to(handlers::channels::add_channel_member))
                            .route("/{channel_id}/members/{user_id}", web::delete().to(handlers::channels::remove_channel_member))
                            .route("/{channel_id}/messages", web::get().to(handlers::messages::list_messages))
                            .route("/{channel_id}/messages", web::post().to(handlers::messages::send_message))
                            .route("/{channel_id}/messages/{message_id}", web::patch().to(handlers::messages::update_message))
                            .route("/{channel_id}/messages/{message_id}", web::delete().to(handlers::messages::delete_message))
                            .route("/{channel_id}/messages/{message_id}/reactions", web::post().to(handlers::messages::add_reaction))
                            .route("/{channel_id}/messages/{message_id}/reactions/{emoji}", web::delete().to(handlers::messages::remove_reaction))
                            .route("/{channel_id}/read", web::post().to(handlers::channels::mark_as_read))
                    )
                    // Files
                    .service(
                        web::scope("/files")
                            .route("/upload", web::post().to(handlers::files::upload_file))
                            .route("/{file_id}", web::get().to(handlers::files::get_file))
                            .route("/{file_id}/download", web::get().to(handlers::files::download_file))
                            .route("/{file_id}", web::delete().to(handlers::files::delete_file))
                    )
                    // Calls
                    .service(
                        web::scope("/calls")
                            .route("", web::post().to(handlers::calls::start_call))
                            .route("/{call_id}", web::get().to(handlers::calls::get_call))
                            .route("/{call_id}/join", web::post().to(handlers::calls::join_call))
                            .route("/{call_id}/leave", web::post().to(handlers::calls::leave_call))
                            .route("/{call_id}/end", web::post().to(handlers::calls::end_call))
                            .route("/{call_id}/participant", web::patch().to(handlers::calls::update_participant))
                    )
                    // Search
                    .service(
                        web::scope("/search")
                            .route("/messages", web::get().to(handlers::search::search_messages))
                    )
                    // Notifications
                    .service(
                        web::scope("/notifications")
                            .route("", web::get().to(handlers::notifications::list_notifications))
                            .route("/{notification_id}/read", web::post().to(handlers::notifications::mark_as_read))
                            .route("/read-all", web::post().to(handlers::notifications::mark_all_as_read))
                    )
            )
            // WebSocket endpoint
            .route("/ws", web::get().to(websocket::ws_handler))
            // Static files for uploads
            .service(Files::new("/uploads", &config.storage.upload_path))
    })
    .bind(&server_addr)?
    .run()
    .await
}
