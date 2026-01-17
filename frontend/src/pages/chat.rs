//! Chat page

use dioxus::prelude::*;
use shared::dto::SendMessageRequest;

use crate::api::ApiClient;
use crate::components::{Avatar, Button};
use crate::state::AppState;
use crate::Route;

#[derive(Props, Clone, PartialEq)]
pub struct ChatPageProps {
    pub channel_id: Option<String>,
}

#[component]
pub fn ChatPage(props: ChatPageProps) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut message_input = use_signal(String::new);
    let mut loading = use_signal(|| false);

    // Load channels on mount
    use_effect(move || {
        spawn(async move {
            if let Ok(channels) = ApiClient::list_channels().await {
                state.write().set_channels(channels);
            }
            if let Ok(teams) = ApiClient::list_teams().await {
                state.write().set_teams(teams);
            }
        });
    });

    // Load messages when channel changes
    let channel_id = props.channel_id.clone();
    use_effect(move || {
        if let Some(cid) = channel_id.clone() {
            spawn(async move {
                if let Ok(messages) = ApiClient::list_messages(&cid, Some(50)).await {
                    if let Ok(uuid) = uuid::Uuid::parse_str(&cid) {
                        state.write().set_messages(uuid, messages);
                    }
                }
            });
        }
    });

    let channel_id_for_send = props.channel_id.clone();
    let send_message = move |_| {
        let content = message_input.read().clone();
        if content.trim().is_empty() {
            return;
        }

        if let Some(cid) = channel_id_for_send.clone() {
            spawn(async move {
                loading.set(true);
                let request = SendMessageRequest {
                    content,
                    message_type: None,
                    reply_to_id: None,
                    attachment_ids: None,
                };

                if let Ok(message) = ApiClient::send_message(&cid, request).await {
                    if let Ok(uuid) = uuid::Uuid::parse_str(&cid) {
                        state.write().add_message(uuid, message);
                    }
                    message_input.set(String::new());
                }
                loading.set(false);
            });
        }
    };

    let current_user = state.read().current_user.clone();
    let channels = state.read().channels.clone();
    let selected_channel = props.channel_id.as_ref().and_then(|cid| {
        uuid::Uuid::parse_str(cid).ok().and_then(|uuid| state.read().get_channel(&uuid).cloned())
    });
    let messages = props.channel_id.as_ref().and_then(|cid| {
        uuid::Uuid::parse_str(cid).ok().and_then(|uuid| state.read().get_messages(&uuid).cloned())
    });

    rsx! {
        div {
            class: "flex h-screen bg-gray-100",
            // Sidebar
            div {
                class: "w-64 bg-gray-800 text-white flex flex-col",
                // User info
                div {
                    class: "p-4 border-b border-gray-700",
                    if let Some(user) = &current_user {
                        div {
                            class: "flex items-center space-x-3",
                            Avatar {
                                name: user.display_name.clone(),
                                src: user.avatar_url.clone().unwrap_or_default(),
                                online: true,
                            }
                            div {
                                p { class: "font-semibold", "{user.display_name}" }
                                p { class: "text-sm text-gray-400", "@{user.username}" }
                            }
                        }
                    }
                }

                // Channels list
                div {
                    class: "flex-1 overflow-y-auto",
                    div {
                        class: "p-4",
                        h3 { class: "text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2", "Channels" }
                        for channel in channels.iter() {
                            Link {
                                to: Route::ChatChannel { channel_id: channel.id.to_string() },
                                class: "block px-3 py-2 rounded hover:bg-gray-700 transition-colors",
                                "# {channel.name}"
                            }
                        }
                    }
                }

                // Bottom actions
                div {
                    class: "p-4 border-t border-gray-700",
                    Link {
                        to: Route::Settings {},
                        class: "block px-3 py-2 rounded hover:bg-gray-700 transition-colors text-gray-400",
                        "⚙️ Settings"
                    }
                }
            }

            // Main content
            div {
                class: "flex-1 flex flex-col",
                // Channel header
                if let Some(channel) = &selected_channel {
                    div {
                        class: "h-16 bg-white border-b flex items-center px-6",
                        h2 { class: "text-xl font-semibold", "# {channel.name}" }
                        if let Some(desc) = &channel.description {
                            p { class: "ml-4 text-gray-500", "{desc}" }
                        }
                    }
                }

                // Messages area
                div {
                    class: "flex-1 overflow-y-auto p-6 space-y-4",
                    if let Some(msgs) = &messages {
                        for msg in msgs.iter() {
                            div {
                                key: "{msg.id}",
                                class: "flex items-start space-x-3",
                                Avatar {
                                    name: msg.sender.display_name.clone(),
                                    src: msg.sender.avatar_url.clone().unwrap_or_default(),
                                }
                                div {
                                    div {
                                        class: "flex items-baseline space-x-2",
                                        span { class: "font-semibold", "{msg.sender.display_name}" }
                                        span { class: "text-xs text-gray-500", "{msg.created_at.format(\"%H:%M\")}" }
                                    }
                                    p { class: "text-gray-800", "{msg.content}" }
                                }
                            }
                        }
                    } else {
                        div {
                            class: "flex items-center justify-center h-full text-gray-500",
                            "Select a channel to start chatting"
                        }
                    }
                }

                // Message input
                if selected_channel.is_some() {
                    div {
                        class: "p-4 bg-white border-t",
                        form {
                            class: "flex space-x-4",
                            prevent_default: "onsubmit",
                            onsubmit: move |_| {
                                send_message(());
                            },
                            input {
                                class: "flex-1 px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "Type a message...",
                                value: "{message_input}",
                                oninput: move |e| message_input.set(e.value()),
                            }
                            Button {
                                button_type: "submit".to_string(),
                                loading: *loading.read(),
                                "Send"
                            }
                        }
                    }
                }
            }
        }
    }
}
