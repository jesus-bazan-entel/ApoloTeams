//! Page components

mod auth;
mod chat;
mod home;
mod settings;
mod team;

pub use auth::{LoginPage, RegisterPage};
pub use chat::ChatPage;
pub use home::HomePage;
pub use settings::SettingsPage;
pub use team::TeamPage;
