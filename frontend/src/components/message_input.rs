//! Message input component

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct MessageInputProps {
    #[props(default)]
    pub placeholder: String,
    pub on_send: EventHandler<String>,
    #[props(default)]
    pub disabled: bool,
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

    let handle_send = {
        let on_send = props.on_send.clone();
        move || {
            let content = message.read().clone();
            if !content.trim().is_empty() {
                on_send.call(content);
                message.set(String::new());
            }
        }
    };

    let handle_send_clone = handle_send.clone();

    rsx! {
        div {
            class: "border-t bg-white p-4",
            form {
                class: "flex items-end space-x-3",
                prevent_default: "onsubmit",
                onsubmit: move |_| {
                    handle_send_clone();
                },
                // Attachment button
                button {
                    r#type: "button",
                    class: "p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors",
                    title: "Attach file",
                    svg {
                        class: "w-5 h-5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"
                        }
                    }
                }

                // Message input area
                div {
                    class: "flex-1 relative",
                    textarea {
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                        placeholder: "{placeholder}",
                        rows: "1",
                        disabled: props.disabled,
                        value: "{message}",
                        oninput: move |e| {
                            message.set(e.value());
                            is_typing.set(!e.value().is_empty());
                        },
                        onkeydown: move |e| {
                            if e.key() == Key::Enter && !e.modifiers().shift() {
                                // Note: In Dioxus 0.5, we handle submit via form onsubmit
                                // The form will handle the actual submission
                            }
                        },
                    }

                    // Typing indicator
                    if *is_typing.read() {
                        div {
                            class: "absolute -top-6 left-0 text-xs text-gray-500",
                            "Typing..."
                        }
                    }
                }

                // Emoji button
                button {
                    r#type: "button",
                    class: "p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors",
                    title: "Add emoji",
                    svg {
                        class: "w-5 h-5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        }
                    }
                }

                // Send button
                button {
                    r#type: "submit",
                    class: "p-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                    disabled: props.disabled || message.read().trim().is_empty(),
                    title: "Send message",
                    svg {
                        class: "w-5 h-5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
                        }
                    }
                }
            }

            // Character count (optional, for long messages)
            if message.read().len() > 500 {
                div {
                    class: "text-xs text-gray-500 mt-1 text-right",
                    "{message.read().len()} / 4000"
                }
            }
        }
    }
}
