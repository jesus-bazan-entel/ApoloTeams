//! Settings page

use dioxus::prelude::*;

use crate::api::ApiClient;
use crate::components::{Avatar, Button};
use crate::state::AppState;
use crate::Route;

#[component]
pub fn SettingsPage() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let navigator = use_navigator();

    let current_user = state.read().current_user.clone();

    let handle_logout = move |_| {
        spawn(async move {
            ApiClient::logout().await;
            state.write().clear_auth();
            navigator.push(Route::Home {});
        });
    };

    rsx! {
        div {
            class: "min-h-screen bg-gray-100",
            // Header
            div {
                class: "bg-white shadow",
                div {
                    class: "container mx-auto px-6 py-4",
                    div {
                        class: "flex items-center justify-between",
                        h1 { class: "text-2xl font-bold", "Settings" }
                        Link {
                            to: Route::Chat {},
                            class: "text-blue-600 hover:underline",
                            "← Back to Chat"
                        }
                    }
                }
            }

            // Content
            div {
                class: "container mx-auto px-6 py-8",
                div {
                    class: "max-w-2xl mx-auto space-y-8",
                    // Profile section
                    div {
                        class: "bg-white rounded-lg shadow p-6",
                        h2 { class: "text-xl font-semibold mb-4", "Profile" }
                        if let Some(user) = &current_user {
                            div {
                                class: "flex items-center space-x-4 mb-6",
                                Avatar {
                                    name: user.display_name.clone(),
                                    src: user.avatar_url.clone().unwrap_or_default(),
                                    size: "xl".to_string(),
                                }
                                div {
                                    h3 { class: "text-lg font-semibold", "{user.display_name}" }
                                    p { class: "text-gray-500", "@{user.username}" }
                                    p { class: "text-gray-500", "{user.email}" }
                                }
                            }
                        }
                        Button {
                            variant: "secondary".to_string(),
                            "Edit Profile"
                        }
                    }

                    // Account section
                    div {
                        class: "bg-white rounded-lg shadow p-6",
                        h2 { class: "text-xl font-semibold mb-4", "Account" }
                        div {
                            class: "space-y-4",
                            Button {
                                variant: "secondary".to_string(),
                                "Change Password"
                            }
                            Button {
                                variant: "danger".to_string(),
                                onclick: handle_logout,
                                "Sign Out"
                            }
                        }
                    }

                    // Notifications section
                    div {
                        class: "bg-white rounded-lg shadow p-6",
                        h2 { class: "text-xl font-semibold mb-4", "Notifications" }
                        div {
                            class: "space-y-4",
                            label {
                                class: "flex items-center space-x-3",
                                input {
                                    r#type: "checkbox",
                                    class: "w-4 h-4 text-blue-600",
                                    checked: true,
                                }
                                span { "Email notifications" }
                            }
                            label {
                                class: "flex items-center space-x-3",
                                input {
                                    r#type: "checkbox",
                                    class: "w-4 h-4 text-blue-600",
                                    checked: true,
                                }
                                span { "Desktop notifications" }
                            }
                            label {
                                class: "flex items-center space-x-3",
                                input {
                                    r#type: "checkbox",
                                    class: "w-4 h-4 text-blue-600",
                                    checked: true,
                                }
                                span { "Sound notifications" }
                            }
                        }
                    }
                }
            }
        }
    }
}
