//! Sidebar component

use dioxus::prelude::*;

use crate::components::Avatar;
use crate::state::AppState;
use shared::models::{Team, Channel};

#[derive(Props, Clone, PartialEq)]
pub struct SidebarProps {
    #[props(default)]
    pub teams: Vec<Team>,
    #[props(default)]
    pub channels: Vec<Channel>,
    #[props(default)]
    pub selected_team_id: Option<String>,
    #[props(default)]
    pub selected_channel_id: Option<String>,
    #[props(default)]
    pub on_team_select: Option<EventHandler<String>>,
    #[props(default)]
    pub on_channel_select: Option<EventHandler<String>>,
}

#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let state = use_context::<Signal<AppState>>();
    let current_user = state.read().current_user.clone();

    rsx! {
        div {
            class: "w-64 bg-gray-900 text-white flex flex-col h-full",
            // User section
            div {
                class: "p-4 border-b border-gray-700",
                if let Some(user) = &current_user {
                    div {
                        class: "flex items-center space-x-3",
                        Avatar {
                            name: user.display_name.clone(),
                            src: user.avatar_url.clone().unwrap_or_default(),
                            size: "md".to_string(),
                        }
                        div {
                            class: "flex-1 min-w-0",
                            p { class: "font-semibold truncate", "{user.display_name}" }
                            p { class: "text-sm text-gray-400 truncate", "@{user.username}" }
                        }
                    }
                }
            }

            // Teams section
            div {
                class: "flex-1 overflow-y-auto",
                // Teams list
                div {
                    class: "p-2",
                    div {
                        class: "flex items-center justify-between px-2 py-1 text-gray-400 text-sm",
                        span { "TEAMS" }
                        button {
                            class: "hover:text-white",
                            title: "Create team",
                            "+"
                        }
                    }
                    for team in props.teams.iter() {
                        button {
                            key: "{team.id}",
                            class: if props.selected_team_id.as_ref() == Some(&team.id.to_string()) {
                                "w-full flex items-center space-x-2 px-2 py-2 rounded bg-gray-700 text-white"
                            } else {
                                "w-full flex items-center space-x-2 px-2 py-2 rounded hover:bg-gray-800 text-gray-300"
                            },
                            onclick: {
                                let team_id = team.id.to_string();
                                let handler = props.on_team_select.clone();
                                move |_| {
                                    if let Some(h) = &handler {
                                        h.call(team_id.clone());
                                    }
                                }
                            },
                            div {
                                class: "w-8 h-8 bg-blue-600 rounded flex items-center justify-center text-sm font-bold",
                                "{team.name.chars().next().unwrap_or('T')}"
                            }
                            span { class: "truncate", "{team.name}" }
                        }
                    }
                }

                // Channels section
                if props.selected_team_id.is_some() {
                    div {
                        class: "p-2 border-t border-gray-700",
                        div {
                            class: "flex items-center justify-between px-2 py-1 text-gray-400 text-sm",
                            span { "CHANNELS" }
                            button {
                                class: "hover:text-white",
                                title: "Create channel",
                                "+"
                            }
                        }
                        for channel in props.channels.iter() {
                            button {
                                key: "{channel.id}",
                                class: if props.selected_channel_id.as_ref() == Some(&channel.id.to_string()) {
                                    "w-full flex items-center space-x-2 px-2 py-2 rounded bg-gray-700 text-white"
                                } else {
                                    "w-full flex items-center space-x-2 px-2 py-2 rounded hover:bg-gray-800 text-gray-300"
                                },
                                onclick: {
                                    let channel_id = channel.id.to_string();
                                    let handler = props.on_channel_select.clone();
                                    move |_| {
                                        if let Some(h) = &handler {
                                            h.call(channel_id.clone());
                                        }
                                    }
                                },
                                span { class: "text-gray-400", "#" }
                                span { class: "truncate", "{channel.name}" }
                            }
                        }
                    }
                }
            }

            // Bottom navigation
            div {
                class: "p-2 border-t border-gray-700",
                Link {
                    to: "/teams",
                    class: "flex items-center space-x-2 px-2 py-2 rounded hover:bg-gray-800 text-gray-300",
                    span { "👥" }
                    span { "Manage Teams" }
                }
                Link {
                    to: "/settings",
                    class: "flex items-center space-x-2 px-2 py-2 rounded hover:bg-gray-800 text-gray-300",
                    span { "⚙️" }
                    span { "Settings" }
                }
            }
        }
    }
}
