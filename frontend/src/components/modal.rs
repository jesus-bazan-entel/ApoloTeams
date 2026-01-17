//! Modal component

use dioxus::prelude::*;

use crate::components::Button;

#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    #[props(default = true)]
    pub open: bool,
    #[props(default)]
    pub title: String,
    #[props(default)]
    pub on_close: Option<EventHandler<()>>,
    pub children: Element,
}

#[component]
pub fn Modal(props: ModalProps) -> Element {
    if !props.open {
        return rsx! {};
    }

    let handle_backdrop_click = {
        let on_close = props.on_close.clone();
        move |_| {
            if let Some(handler) = &on_close {
                handler.call(());
            }
        }
    };

    let handle_escape = {
        let on_close = props.on_close.clone();
        move |e: Event<KeyboardData>| {
            if e.key() == Key::Escape {
                if let Some(handler) = &on_close {
                    handler.call(());
                }
            }
        }
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 overflow-y-auto",
            onkeydown: handle_escape,
            // Backdrop
            div {
                class: "fixed inset-0 bg-black bg-opacity-50 transition-opacity",
                onclick: handle_backdrop_click,
            }

            // Modal container
            div {
                class: "flex min-h-full items-center justify-center p-4",
                // Modal content
                div {
                    class: "relative bg-white rounded-lg shadow-xl max-w-lg w-full transform transition-all",
                    onclick: move |e| e.stop_propagation(),
                    // Header
                    if !props.title.is_empty() {
                        div {
                            class: "flex items-center justify-between px-6 py-4 border-b",
                            h3 {
                                class: "text-lg font-semibold",
                                "{props.title}"
                            }
                            button {
                                class: "text-gray-400 hover:text-gray-600 transition-colors",
                                onclick: {
                                    let on_close = props.on_close.clone();
                                    move |_| {
                                        if let Some(handler) = &on_close {
                                            handler.call(());
                                        }
                                    }
                                },
                                "✕"
                            }
                        }
                    }

                    // Body
                    div {
                        class: "px-6 py-4",
                        {props.children}
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ConfirmModalProps {
    #[props(default = true)]
    pub open: bool,
    #[props(default)]
    pub title: String,
    #[props(default)]
    pub message: String,
    #[props(default = "Confirm".to_string())]
    pub confirm_text: String,
    #[props(default = "Cancel".to_string())]
    pub cancel_text: String,
    #[props(default = "danger".to_string())]
    pub confirm_variant: String,
    #[props(default)]
    pub on_confirm: Option<EventHandler<()>>,
    #[props(default)]
    pub on_cancel: Option<EventHandler<()>>,
}

#[component]
pub fn ConfirmModal(props: ConfirmModalProps) -> Element {
    let handle_cancel = {
        let on_cancel = props.on_cancel.clone();
        move |_| {
            if let Some(handler) = &on_cancel {
                handler.call(());
            }
        }
    };

    let handle_confirm = {
        let on_confirm = props.on_confirm.clone();
        move |_| {
            if let Some(handler) = &on_confirm {
                handler.call(());
            }
        }
    };

    rsx! {
        Modal {
            open: props.open,
            title: props.title.clone(),
            on_close: props.on_cancel.clone(),
            div {
                class: "space-y-4",
                p { class: "text-gray-600", "{props.message}" }
                div {
                    class: "flex justify-end space-x-3",
                    Button {
                        variant: "secondary".to_string(),
                        onclick: handle_cancel,
                        "{props.cancel_text}"
                    }
                    Button {
                        variant: props.confirm_variant.clone(),
                        onclick: handle_confirm,
                        "{props.confirm_text}"
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertModalProps {
    #[props(default = true)]
    pub open: bool,
    #[props(default)]
    pub title: String,
    #[props(default)]
    pub message: String,
    #[props(default = "info".to_string())]
    pub variant: String, // info, success, warning, error
    #[props(default)]
    pub on_close: Option<EventHandler<()>>,
}

#[component]
pub fn AlertModal(props: AlertModalProps) -> Element {
    let icon = match props.variant.as_str() {
        "success" => "✓",
        "warning" => "⚠",
        "error" => "✕",
        _ => "ℹ",
    };

    let icon_color = match props.variant.as_str() {
        "success" => "text-green-500",
        "warning" => "text-yellow-500",
        "error" => "text-red-500",
        _ => "text-blue-500",
    };

    rsx! {
        Modal {
            open: props.open,
            title: props.title.clone(),
            on_close: props.on_close.clone(),
            div {
                class: "text-center space-y-4",
                div {
                    class: "text-4xl {icon_color}",
                    "{icon}"
                }
                p { class: "text-gray-600", "{props.message}" }
                Button {
                    onclick: {
                        let on_close = props.on_close.clone();
                        move |_| {
                            if let Some(handler) = &on_close {
                                handler.call(());
                            }
                        }
                    },
                    "OK"
                }
            }
        }
    }
}
