//! Button component

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    pub children: Element,
    #[props(default = "primary".to_string())]
    pub variant: String,
    #[props(default = "md".to_string())]
    pub size: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub loading: bool,
    #[props(default = "button".to_string())]
    pub button_type: String,
    pub onclick: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let base_class = "inline-flex items-center justify-center font-medium rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2";

    let variant_class = match props.variant.as_str() {
        "secondary" => "bg-gray-200 text-gray-900 hover:bg-gray-300 focus:ring-gray-500",
        "danger" => "bg-red-600 text-white hover:bg-red-700 focus:ring-red-500",
        "ghost" => "bg-transparent text-gray-700 hover:bg-gray-100 focus:ring-gray-500",
        "link" => "bg-transparent text-blue-600 hover:underline focus:ring-blue-500",
        _ => "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500",
    };

    let size_class = match props.size.as_str() {
        "sm" => "px-3 py-1.5 text-sm",
        "lg" => "px-6 py-3 text-lg",
        _ => "px-4 py-2 text-base",
    };

    let disabled_class = if props.disabled || props.loading {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    };

    rsx! {
        button {
            r#type: "{props.button_type}",
            class: "{base_class} {variant_class} {size_class} {disabled_class}",
            disabled: props.disabled || props.loading,
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },
            if props.loading {
                svg {
                    class: "animate-spin -ml-1 mr-2 h-4 w-4",
                    xmlns: "http://www.w3.org/2000/svg",
                    fill: "none",
                    view_box: "0 0 24 24",
                    circle {
                        class: "opacity-25",
                        cx: "12",
                        cy: "12",
                        r: "10",
                        stroke: "currentColor",
                        stroke_width: "4",
                    }
                    path {
                        class: "opacity-75",
                        fill: "currentColor",
                        d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                    }
                }
            }
            {props.children}
        }
    }
}
