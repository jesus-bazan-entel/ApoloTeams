//! User status component

use dioxus::prelude::*;

use shared::models::UserStatus;

#[derive(Props, Clone, PartialEq)]
pub struct StatusIndicatorProps {
    pub status: UserStatus,
    #[props(default = "md".to_string())]
    pub size: String,
}

#[component]
pub fn StatusIndicator(props: StatusIndicatorProps) -> Element {
    let (color, title) = match props.status {
        UserStatus::Online => ("bg-green-500", "Online"),
        UserStatus::Away => ("bg-yellow-500", "Away"),
        UserStatus::Busy => ("bg-red-500", "Busy"),
        UserStatus::DoNotDisturb => ("bg-red-600", "Do Not Disturb"),
        UserStatus::Offline => ("bg-gray-400", "Offline"),
    };

    let size_class = match props.size.as_str() {
        "sm" => "w-2 h-2",
        "lg" => "w-4 h-4",
        _ => "w-3 h-3",
    };

    rsx! {
        span {
            class: "{size_class} {color} rounded-full inline-block",
            title: "{title}",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct StatusSelectorProps {
    pub current_status: UserStatus,
    #[props(default)]
    pub on_change: Option<EventHandler<UserStatus>>,
}

#[component]
pub fn StatusSelector(props: StatusSelectorProps) -> Element {
    let mut is_open = use_signal(|| false);

    let statuses = vec![
        (UserStatus::Online, "Online", "bg-green-500"),
        (UserStatus::Away, "Away", "bg-yellow-500"),
        (UserStatus::Busy, "Busy", "bg-red-500"),
        (UserStatus::DoNotDisturb, "Do Not Disturb", "bg-red-600"),
        (UserStatus::Offline, "Appear Offline", "bg-gray-400"),
    ];

    let current = statuses
        .iter()
        .find(|(s, _, _)| *s == props.current_status)
        .unwrap_or(&statuses[0]);

    rsx! {
        div {
            class: "relative",
            // Current status button
            button {
                class: "flex items-center space-x-2 px-3 py-2 rounded-lg hover:bg-gray-100 transition-colors",
                onclick: move |_| {
                    let current_value = *is_open.read();
                    is_open.set(!current_value);
                },
                span {
                    class: "w-3 h-3 {current.2} rounded-full",
                }
                span { "{current.1}" }
                span { class: "text-gray-400", "▼" }
            }

            // Dropdown
            if *is_open.read() {
                div {
                    class: "absolute top-full left-0 mt-1 w-48 bg-white rounded-lg shadow-lg border z-50",
                    for (status, label, color) in statuses.iter() {
                        button {
                            key: "{label}",
                            class: "w-full flex items-center space-x-2 px-3 py-2 hover:bg-gray-100 transition-colors first:rounded-t-lg last:rounded-b-lg",
                            onclick: {
                                let status = status.clone();
                                let handler = props.on_change.clone();
                                move |_| {
                                    if let Some(h) = &handler {
                                        h.call(status.clone());
                                    }
                                    is_open.set(false);
                                }
                            },
                            span {
                                class: "w-3 h-3 {color} rounded-full",
                            }
                            span { "{label}" }
                            if *status == props.current_status {
                                span { class: "ml-auto text-blue-500", "✓" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct UserPresenceProps {
    pub username: String,
    pub display_name: String,
    pub status: UserStatus,
    #[props(default)]
    pub status_message: Option<String>,
    #[props(default)]
    pub avatar_url: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<()>>,
}

#[component]
pub fn UserPresence(props: UserPresenceProps) -> Element {
    use crate::components::Avatar;

    rsx! {
        button {
            class: "flex items-center space-x-3 p-2 rounded-lg hover:bg-gray-100 transition-colors w-full text-left",
            onclick: {
                let handler = props.on_click.clone();
                move |_| {
                    if let Some(h) = &handler {
                        h.call(());
                    }
                }
            },
            // Avatar with status indicator
            div {
                class: "relative",
                Avatar {
                    name: props.display_name.clone(),
                    src: props.avatar_url.clone().unwrap_or_default(),
                    size: "md".to_string(),
                }
                div {
                    class: "absolute bottom-0 right-0 transform translate-x-1 translate-y-1",
                    StatusIndicator {
                        status: props.status.clone(),
                        size: "sm".to_string(),
                    }
                }
            }

            // User info
            div {
                class: "flex-1 min-w-0",
                p {
                    class: "font-medium truncate",
                    "{props.display_name}"
                }
                if let Some(message) = &props.status_message {
                    p {
                        class: "text-sm text-gray-500 truncate",
                        "{message}"
                    }
                } else {
                    p {
                        class: "text-sm text-gray-500",
                        "@{props.username}"
                    }
                }
            }
        }
    }
}
