//! Avatar component

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct AvatarProps {
    #[props(default = String::new())]
    pub src: String,
    pub name: String,
    #[props(default = "md".to_string())]
    pub size: String,
    #[props(default = false)]
    pub online: bool,
}

#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let size_class = match props.size.as_str() {
        "sm" => "w-8 h-8 text-xs",
        "lg" => "w-16 h-16 text-xl",
        "xl" => "w-24 h-24 text-3xl",
        _ => "w-10 h-10 text-sm",
    };

    let initials: String = props
        .name
        .split_whitespace()
        .take(2)
        .map(|s| s.chars().next().unwrap_or_default())
        .collect::<String>()
        .to_uppercase();

    let bg_color = get_color_from_name(&props.name);

    rsx! {
        div {
            class: "relative inline-block",
            if !props.src.is_empty() {
                img {
                    class: "{size_class} rounded-full object-cover",
                    src: "{props.src}",
                    alt: "{props.name}",
                }
            } else {
                div {
                    class: "{size_class} rounded-full flex items-center justify-center font-semibold text-white {bg_color}",
                    "{initials}"
                }
            }
            if props.online {
                span {
                    class: "absolute bottom-0 right-0 w-3 h-3 bg-green-500 border-2 border-white rounded-full",
                }
            }
        }
    }
}

fn get_color_from_name(name: &str) -> &'static str {
    let hash: u32 = name.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
    let colors = [
        "bg-red-500",
        "bg-orange-500",
        "bg-amber-500",
        "bg-yellow-500",
        "bg-lime-500",
        "bg-green-500",
        "bg-emerald-500",
        "bg-teal-500",
        "bg-cyan-500",
        "bg-sky-500",
        "bg-blue-500",
        "bg-indigo-500",
        "bg-violet-500",
        "bg-purple-500",
        "bg-fuchsia-500",
        "bg-pink-500",
    ];
    colors[(hash as usize) % colors.len()]
}
