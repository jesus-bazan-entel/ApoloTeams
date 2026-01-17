//! Message input component

use dioxus::prelude::*;

use crate::components::Button;

#[derive(Props, Clone, PartialEq)]
pub struct MessageInputProps {
    #[props(default)]
    pub placeholder: String,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub on_send: Option<EventHandler<String>>,
    #[props(default)]
    pub on_typing: Option<EventHandler<()>>,
    #[props(default)]
    pub on_file_attach: Option<EventHandler<()>>,
}

#[component]
pub fn MessageInput(props: MessageInputProps) -> Element {
    let mut message = use_signal(String::new);
    let mut is_typing = use_signal(|| false);

    let placeholder = if props.placeholder.is_empty() {
        "Type a message...".to_string()
    } else {
        props.placeholder.clone()
    };

    let handle_submit = {
        let on_send = props.on_send.clone();
        move |e: Event<FormData>| {
            e.prevent_default();
            let content = message.read().trim().to_string();
            if !content.is_empty() {
                if let Some(handler) = &on_send {
                    handler.call(content);
                }
                message.set(String::new());
            }
        }
    };

    let handle_input = {
        let on_typing = props.on_typing.clone();
        move |e: Event<FormData>| {
            message.set(e.value().clone());
            if !*is_typing.read() {
                is_typing.set(true);
                if let Some(handler) = &on_typing {
                    handler.call(());
                }
            }
        }
    };

    let handle_keydown = {
        let on_send = props.on_send.clone();
        move |e: Event<KeyboardData>| {
            if e.key() == Key::Enter && !e.modifiers().shift() {
                e.prevent_default();
                let content = message.read().trim().to_string();
                if !content.is_empty() {
                    if let Some(handler) = &on_send {
                        handler.call(content);
                    }
                    message.set(String::new());
                }
            }
        }
    };

    rsx! {
        div {
            class: "border-t bg-white p-4",
            form {
                onsubmit: handle_submit,
                class: "flex items-end space-x-3",
                // Attachment button
                button {
                    r#type: "button",
                    class: "p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors",
                    title: "Attach file",
                    onclick: {
                        let handler = props.on_file_attach.clone();
                        move |_| {
                            if let Some(h) = &handler {
                                h.call(());
                            }
                        }
                    },
                    "📎"
                }

                // Message input
                div {
                    class: "flex-1",
                    textarea {
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                        placeholder: "{placeholder}",
                        rows: "1",
                        disabled: props.disabled,
                        value: "{message}",
                        oninput: handle_input,
                        onkeydown: handle_keydown,
                    }
                }

                // Emoji button
                button {
                    r#type: "button",
                    class: "p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors",
                    title: "Add emoji",
                    "😀"
                }

                // Send button
                Button {
                    r#type: "submit".to_string(),
                    disabled: props.disabled || message.read().trim().is_empty(),
                    "Send"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TypingIndicatorProps {
    #[props(default)]
    pub users: Vec<String>,
}

#[component]
pub fn TypingIndicator(props: TypingIndicatorProps) -> Element {
    if props.users.is_empty() {
        return rsx! {};
    }

    let text = match props.users.len() {
        1 => format!("{} is typing...", props.users[0]),
        2 => format!("{} and {} are typing...", props.users[0], props.users[1]),
        _ => format!("{} and {} others are typing...", props.users[0], props.users.len() - 1),
    };

    rsx! {
        div {
            class: "px-4 py-2 text-sm text-gray-500 italic",
            "{text}"
        }
    }
}
