//! Input component

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    #[props(default = "text".to_string())]
    pub input_type: String,
    pub value: String,
    pub onchange: EventHandler<String>,
    #[props(default = String::new())]
    pub placeholder: String,
    #[props(default = String::new())]
    pub label: String,
    #[props(default = String::new())]
    pub error: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
}

#[component]
pub fn Input(props: InputProps) -> Element {
    let has_error = !props.error.is_empty();
    let border_class = if has_error {
        "border-red-500 focus:ring-red-500 focus:border-red-500"
    } else {
        "border-gray-300 focus:ring-blue-500 focus:border-blue-500"
    };

    rsx! {
        div {
            class: "w-full",
            if !props.label.is_empty() {
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "{props.label}"
                    if props.required {
                        span { class: "text-red-500 ml-1", "*" }
                    }
                }
            }
            input {
                r#type: "{props.input_type}",
                class: "w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 {border_class}",
                placeholder: "{props.placeholder}",
                value: "{props.value}",
                disabled: props.disabled,
                required: props.required,
                oninput: move |e| props.onchange.call(e.value()),
            }
            if has_error {
                p {
                    class: "mt-1 text-sm text-red-500",
                    "{props.error}"
                }
            }
        }
    }
}
