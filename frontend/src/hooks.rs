//! Custom hooks for the Rust Teams frontend

use dioxus::prelude::*;
use crate::state::AppState;
use shared::dto::UserResponse;

/// Hook to get the current authenticated user
pub fn use_current_user() -> Option<UserResponse> {
    let state = use_context::<Signal<AppState>>();
    let result = state.read().current_user.clone();
    result
}

/// Hook to check if user is authenticated
pub fn use_is_authenticated() -> bool {
    let state = use_context::<Signal<AppState>>();
    let result = state.read().is_authenticated();
    result
}

/// Hook to get the auth token
pub fn use_auth_token() -> Option<String> {
    let state = use_context::<Signal<AppState>>();
    let result = state.read().access_token.clone();
    result
}
