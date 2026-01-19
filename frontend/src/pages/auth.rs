//! Authentication pages

use dioxus::prelude::*;
use shared::dto::{LoginRequest, RegisterRequest};

use crate::api::ApiClient;
use crate::components::{Button, Input};
use crate::state::AppState;
use crate::Route;

#[component]
pub fn LoginPage() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let navigator = use_navigator();

    let handle_login = move |_| {
        let email_val = email.read().clone();
        let password_val = password.read().clone();

        spawn(async move {
            loading.set(true);
            error.set(None);

            let request = LoginRequest {
                email: email_val,
                password: password_val,
            };

            match ApiClient::login(request).await {
                Ok(response) => {
                    state.write().set_auth(
                        response.user,
                        response.access_token,
                        response.refresh_token,
                    );
                    navigator.push(Route::Chat {});
                }
                Err(e) => {
                    error.set(Some(e));
                }
            }

            loading.set(false);
        });
    };

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center bg-gray-100",
            div {
                class: "max-w-md w-full bg-white rounded-lg shadow-md p-8",
                div {
                    class: "text-center mb-8",
                    h1 {
                        class: "text-3xl font-bold text-gray-900",
                        "Rust Teams"
                    }
                    p {
                        class: "text-gray-600 mt-2",
                        "Sign in to your account"
                    }
                }

                if let Some(err) = error.read().as_ref() {
                    div {
                        class: "mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded",
                        "{err}"
                    }
                }

                form {
                    class: "space-y-4",
                    prevent_default: "onsubmit",
                    onsubmit: move |_| {
                        handle_login(());
                    },
                    Input {
                        input_type: "email".to_string(),
                        label: "Email".to_string(),
                        placeholder: "you@example.com".to_string(),
                        value: email.read().clone(),
                        onchange: move |v| email.set(v),
                        required: true,
                    }
                    Input {
                        input_type: "password".to_string(),
                        label: "Password".to_string(),
                        placeholder: "••••••••".to_string(),
                        value: password.read().clone(),
                        onchange: move |v| password.set(v),
                        required: true,
                    }
                    Button {
                        button_type: "submit".to_string(),
                        loading: *loading.read(),
                        "Sign In"
                    }
                }

                div {
                    class: "mt-6 text-center",
                    p {
                        class: "text-gray-600",
                        "Don't have an account? "
                        Link {
                            to: Route::Register {},
                            class: "text-blue-600 hover:underline",
                            "Sign up"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn RegisterPage() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut email = use_signal(String::new);
    let mut username = use_signal(String::new);
    let mut display_name = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let navigator = use_navigator();

    let mut handle_register = move |_| {
        let email_val = email.read().clone();
        let username_val = username.read().clone();
        let display_name_val = display_name.read().clone();
        let password_val = password.read().clone();
        let confirm_password_val = confirm_password.read().clone();

        if password_val != confirm_password_val {
            error.set(Some("Passwords do not match".to_string()));
            return;
        }

        spawn(async move {
            loading.set(true);
            error.set(None);

            let request = RegisterRequest {
                email: email_val,
                username: username_val,
                display_name: display_name_val,
                password: password_val,
            };

            match ApiClient::register(request).await {
                Ok(response) => {
                    state.write().set_auth(
                        response.user,
                        response.access_token,
                        response.refresh_token,
                    );
                    navigator.push(Route::Chat {});
                }
                Err(e) => {
                    error.set(Some(e));
                }
            }

            loading.set(false);
        });
    };

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center bg-gray-100",
            div {
                class: "max-w-md w-full bg-white rounded-lg shadow-md p-8",
                div {
                    class: "text-center mb-8",
                    h1 {
                        class: "text-3xl font-bold text-gray-900",
                        "Create Account"
                    }
                    p {
                        class: "text-gray-600 mt-2",
                        "Join Rust Teams today"
                    }
                }

                if let Some(err) = error.read().as_ref() {
                    div {
                        class: "mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded",
                        "{err}"
                    }
                }

                form {
                    class: "space-y-4",
                    prevent_default: "onsubmit",
                    onsubmit: move |_| {
                        handle_register(());
                    },
                    Input {
                        input_type: "email".to_string(),
                        label: "Email".to_string(),
                        placeholder: "you@example.com".to_string(),
                        value: email.read().clone(),
                        onchange: move |v| email.set(v),
                        required: true,
                    }
                    Input {
                        input_type: "text".to_string(),
                        label: "Username".to_string(),
                        placeholder: "johndoe".to_string(),
                        value: username.read().clone(),
                        onchange: move |v| username.set(v),
                        required: true,
                    }
                    Input {
                        input_type: "text".to_string(),
                        label: "Display Name".to_string(),
                        placeholder: "John Doe".to_string(),
                        value: display_name.read().clone(),
                        onchange: move |v| display_name.set(v),
                        required: true,
                    }
                    Input {
                        input_type: "password".to_string(),
                        label: "Password".to_string(),
                        placeholder: "••••••••".to_string(),
                        value: password.read().clone(),
                        onchange: move |v| password.set(v),
                        required: true,
                    }
                    Input {
                        input_type: "password".to_string(),
                        label: "Confirm Password".to_string(),
                        placeholder: "••••••••".to_string(),
                        value: confirm_password.read().clone(),
                        onchange: move |v| confirm_password.set(v),
                        required: true,
                    }
                    Button {
                        button_type: "submit".to_string(),
                        loading: *loading.read(),
                        "Create Account"
                    }
                }

                div {
                    class: "mt-6 text-center",
                    p {
                        class: "text-gray-600",
                        "Already have an account? "
                        Link {
                            to: Route::Login {},
                            class: "text-blue-600 hover:underline",
                            "Sign in"
                        }
                    }
                }
            }
        }
    }
}
