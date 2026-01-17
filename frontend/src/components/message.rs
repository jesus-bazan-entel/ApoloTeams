//! Message component

use dioxus::prelude::*;

use crate::components::Avatar;
use shared::models::Message;

#[derive(Props, Clone, PartialEq)]
pub struct MessageProps {
    pub message: Message,
    #[props(default)]
    pub is_own: bool,
    #[props(default)]
    pub show_avatar: bool,
    #[props(default)]
    pub on_reply: Option<EventHandler<String>>,
    #[props(default)]
    pub on_react: Option<EventHandler<(String, String)>>,
}

#[component]
pub fn MessageComponent(props: MessageProps) -> Element {
    let message = &props.message;
    let formatted_time = message.created_at.format("%H:%M").to_string();

    rsx! {
        div {
            class: if props.is_own {
                "flex items-start space-x-3 p-2 hover:bg-gray-50 flex-row-reverse space-x-reverse"
            } else {
                "flex items-start space-x-3 p-2 hover:bg-gray-50"
            },
            // Avatar
            if props.show_avatar {
                Avatar {
                    name: message.sender_id.to_string(),
                    size: "sm".to_string(),
                }
            } else {
                div { class: "w-8" }
            }

            // Message content
            div {
                class: if props.is_own { "flex-1 text-right" } else { "flex-1" },
                // Header
                if props.show_avatar {
                    div {
                        class: "flex items-center space-x-2 mb-1",
                        span {
                            class: "font-semibold text-sm",
                            "User"
                        }
                        span {
                            class: "text-xs text-gray-500",
                            "{formatted_time}"
                        }
                        if message.is_edited {
                            span {
                                class: "text-xs text-gray-400",
                                "(edited)"
                            }
                        }
                    }
                }

                // Message body
                div {
                    class: if props.is_own {
                        "inline-block bg-blue-500 text-white rounded-lg px-4 py-2 max-w-md"
                    } else {
                        "inline-block bg-gray-100 rounded-lg px-4 py-2 max-w-md"
                    },
                    p { class: "whitespace-pre-wrap break-words", "{message.content}" }
                }

                // Attachments
                if !message.attachments.is_empty() {
                    div {
                        class: "mt-2 space-y-2",
                        for attachment in message.attachments.iter() {
                            div {
                                key: "{attachment}",
                                class: "flex items-center space-x-2 text-sm text-blue-600",
                                span { "📎" }
                                a {
                                    href: "{attachment}",
                                    class: "hover:underline",
                                    "{attachment}"
                                }
                            }
                        }
                    }
                }

                // Reactions
                if !message.reactions.is_empty() {
                    div {
                        class: "mt-2 flex flex-wrap gap-1",
                        for (emoji, count) in message.reactions.iter() {
                            button {
                                key: "{emoji}",
                                class: "inline-flex items-center space-x-1 px-2 py-1 bg-gray-100 rounded-full text-sm hover:bg-gray-200",
                                onclick: {
                                    let msg_id = message.id.to_string();
                                    let emoji = emoji.clone();
                                    let handler = props.on_react.clone();
                                    move |_| {
                                        if let Some(h) = &handler {
                                            h.call((msg_id.clone(), emoji.clone()));
                                        }
                                    }
                                },
                                span { "{emoji}" }
                                span { class: "text-gray-500", "{count}" }
                            }
                        }
                    }
                }

                // Actions (on hover)
                div {
                    class: "mt-1 opacity-0 group-hover:opacity-100 transition-opacity",
                    div {
                        class: "flex items-center space-x-2 text-gray-400",
                        button {
                            class: "hover:text-gray-600",
                            title: "React",
                            "😀"
                        }
                        button {
                            class: "hover:text-gray-600",
                            title: "Reply",
                            onclick: {
                                let msg_id = message.id.to_string();
                                let handler = props.on_reply.clone();
                                move |_| {
                                    if let Some(h) = &handler {
                                        h.call(msg_id.clone());
                                    }
                                }
                            },
                            "↩️"
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MessageListProps {
    pub messages: Vec<Message>,
    #[props(default)]
    pub current_user_id: Option<String>,
    #[props(default)]
    pub on_reply: Option<EventHandler<String>>,
    #[props(default)]
    pub on_react: Option<EventHandler<(String, String)>>,
}

#[component]
pub fn MessageList(props: MessageListProps) -> Element {
    rsx! {
        div {
            class: "flex-1 overflow-y-auto p-4 space-y-1",
            for (i, message) in props.messages.iter().enumerate() {
                {
                    let show_avatar = i == 0 || {
                        let prev = &props.messages[i - 1];
                        prev.sender_id != message.sender_id ||
                        (message.created_at - prev.created_at).num_minutes() > 5
                    };
                    let is_own = props.current_user_id.as_ref()
                        .map(|id| id == &message.sender_id.to_string())
                        .unwrap_or(false);

                    rsx! {
                        MessageComponent {
                            key: "{message.id}",
                            message: message.clone(),
                            is_own: is_own,
                            show_avatar: show_avatar,
                            on_reply: props.on_reply.clone(),
                            on_react: props.on_react.clone(),
                        }
                    }
                }
            }
        }
    }
}
