//! Service layer containing business logic

pub mod auth;
pub mod users;
pub mod teams;
pub mod channels;
pub mod messages;
pub mod files;
pub mod calls;
pub mod notifications;
pub mod meetings;

use sqlx::PgPool;
use std::sync::Arc;

use crate::config::AppConfig;

/// Container for all application services
pub struct Services {
    pub auth: auth::AuthService,
    pub users: users::UserService,
    pub teams: teams::TeamService,
    pub channels: channels::ChannelService,
    pub messages: messages::MessageService,
    pub files: files::FileService,
    pub calls: calls::CallService,
    pub notifications: notifications::NotificationService,
    pub meetings: meetings::MeetingService,
}

impl Services {
    pub fn new(pool: PgPool, config: AppConfig) -> Self {
        let pool = Arc::new(pool);
        let config = Arc::new(config);

        Self {
            auth: auth::AuthService::new(pool.clone(), config.clone()),
            users: users::UserService::new(pool.clone()),
            teams: teams::TeamService::new(pool.clone()),
            channels: channels::ChannelService::new(pool.clone()),
            messages: messages::MessageService::new(pool.clone()),
            files: files::FileService::new(pool.clone(), config.clone()),
            calls: calls::CallService::new(pool.clone()),
            notifications: notifications::NotificationService::new(pool.clone()),
            meetings: meetings::MeetingService::new(pool.clone()),
        }
    }
}
