//! Page components

pub mod auth;
pub mod chat;
pub mod home;
pub mod settings;
pub mod team;

pub use auth::{LoginPage, RegisterPage};
pub use chat::ChatPage;
pub use home::HomePage;
pub use settings::SettingsPage;
pub use team::TeamPage;
