//! Rust Teams Frontend - Dioxus Web Application

use dioxus::prelude::*;

mod api;
mod components;
mod hooks;
mod pages;
mod state;
mod websocket;

use state::AppState;

fn main() {
    // Initialize logging
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");

    log::info!("Starting Rust Teams frontend");

    // Launch the Dioxus app
    dioxus::launch(App);
}

/// Application routes
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
    #[route("/chat")]
    Chat {},
    #[route("/chat/:channel_id")]
    ChatChannel { channel_id: String },
    #[route("/teams/:team_id")]
    Team { team_id: String },
    #[route("/settings")]
    Settings {},
}

fn App() -> Element {
    // Initialize global state
    use_context_provider(|| Signal::new(AppState::default()));

    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    let state = use_context::<Signal<AppState>>();

    // Check if user is authenticated
    if state.read().is_authenticated() {
        rsx! {
            pages::home::HomePage {}
        }
    } else {
        rsx! {
            pages::auth::LoginPage {}
        }
    }
}

#[component]
fn Login() -> Element {
    rsx! {
        pages::auth::LoginPage {}
    }
}

#[component]
fn Register() -> Element {
    rsx! {
        pages::auth::RegisterPage {}
    }
}

#[component]
fn Chat() -> Element {
    rsx! {
        pages::chat::ChatPage { channel_id: None }
    }
}

#[component]
fn ChatChannel(channel_id: String) -> Element {
    rsx! {
        pages::chat::ChatPage { channel_id: Some(channel_id) }
    }
}

#[component]
fn Team(team_id: String) -> Element {
    rsx! {
        pages::team::TeamPage { team_id }
    }
}

#[component]
fn Settings() -> Element {
    rsx! {
        pages::settings::SettingsPage {}
    }
}
