//! Team management page

use dioxus::prelude::*;

use crate::api::ApiClient;
use crate::components::{Button, Input};
use crate::state::AppState;
use crate::Route;
use shared::dto::CreateTeamRequest;

#[derive(Props, Clone, PartialEq)]
pub struct TeamPageProps {
    pub team_id: String,
}

#[component]
pub fn TeamPage(props: TeamPageProps) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut show_create_modal = use_signal(|| false);
    let mut team_name = use_signal(String::new);
    let mut team_description = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    let teams = state.read().teams.clone();

    let mut handle_create_team = move |_| {
        let name = team_name.read().clone();
        let description = team_description.read().clone();

        if name.is_empty() {
            error.set(Some("Team name is required".to_string()));
            return;
        }

        loading.set(true);
        error.set(None);

        spawn(async move {
            let request = CreateTeamRequest {
                name,
                description: if description.is_empty() {
                    None
                } else {
                    Some(description)
                },
                avatar_url: None,
            };

            match ApiClient::create_team(request).await {
                Ok(_team) => {
                    show_create_modal.set(false);
                    team_name.set(String::new());
                    team_description.set(String::new());
                    // Refresh teams list
                    if let Ok(teams) = ApiClient::list_teams().await {
                        state.write().set_teams(teams);
                    }
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                }
            }
            loading.set(false);
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
                        h1 { class: "text-2xl font-bold", "Teams" }
                        div {
                            class: "flex items-center space-x-4",
                            Button {
                                onclick: move |_| show_create_modal.set(true),
                                "Create Team"
                            }
                            Link {
                                to: Route::Chat {},
                                class: "text-blue-600 hover:underline",
                                "← Back to Chat"
                            }
                        }
                    }
                }
            }

            // Content
            div {
                class: "container mx-auto px-6 py-8",
                div {
                    class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                    for team in teams.iter() {
                        div {
                            key: "{team.id}",
                            class: "bg-white rounded-lg shadow p-6 hover:shadow-lg transition-shadow",
                            div {
                                class: "flex items-center space-x-4 mb-4",
                                div {
                                    class: "w-12 h-12 bg-blue-500 rounded-lg flex items-center justify-center text-white text-xl font-bold",
                                    "{team.name.chars().next().unwrap_or('T')}"
                                }
                                div {
                                    h3 { class: "text-lg font-semibold", "{team.name}" }
                                    if let Some(desc) = &team.description {
                                        p { class: "text-gray-500 text-sm", "{desc}" }
                                    }
                                }
                            }
                            div {
                                class: "flex items-center justify-between text-sm text-gray-500",
                                span { "Created: {team.created_at.format(\"%b %d, %Y\")}" }
                                Link {
                                    to: Route::Chat {},
                                    class: "text-blue-600 hover:underline",
                                    "Open →"
                                }
                            }
                        }
                    }
                }

                if teams.is_empty() {
                    div {
                        class: "text-center py-20",
                        div {
                            class: "text-6xl mb-4",
                            "👥"
                        }
                        h2 { class: "text-2xl font-semibold mb-2", "No teams yet" }
                        p { class: "text-gray-500 mb-6", "Create your first team to get started" }
                        Button {
                            onclick: move |_| show_create_modal.set(true),
                            "Create Team"
                        }
                    }
                }
            }

            // Create Team Modal
            if *show_create_modal.read() {
                div {
                    class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                    onclick: move |_| show_create_modal.set(false),
                    div {
                        class: "bg-white rounded-lg shadow-xl p-6 w-full max-w-md",
                        onclick: move |e| e.stop_propagation(),
                        h2 { class: "text-xl font-semibold mb-4", "Create New Team" }

                        if let Some(err) = error.read().as_ref() {
                            div {
                                class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4",
                                "{err}"
                            }
                        }

                        form {
                            prevent_default: "onsubmit",
                            onsubmit: move |_| {
                                handle_create_team(());
                            },
                            div {
                                class: "space-y-4",
                                Input {
                                    label: "Team Name".to_string(),
                                    value: team_name.read().clone(),
                                    onchange: move |v| team_name.set(v),
                                    placeholder: "Enter team name".to_string(),
                                    required: true,
                                }
                                div {
                                    label {
                                        class: "block text-sm font-medium text-gray-700 mb-1",
                                        "Description (optional)"
                                    }
                                    textarea {
                                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                        rows: "3",
                                        placeholder: "Enter team description",
                                        value: "{team_description}",
                                        oninput: move |e| team_description.set(e.value()),
                                    }
                                }
                            }
                            div {
                                class: "flex justify-end space-x-3 mt-6",
                                Button {
                                    variant: "secondary".to_string(),
                                    onclick: move |_| show_create_modal.set(false),
                                    "Cancel"
                                }
                                Button {
                                    button_type: "submit".to_string(),
                                    disabled: *loading.read(),
                                    if *loading.read() { "Creating..." } else { "Create Team" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
