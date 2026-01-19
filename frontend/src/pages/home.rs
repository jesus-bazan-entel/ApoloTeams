//! Home page

use dioxus::prelude::*;

use crate::state::AppState;
use crate::Route;

#[component]
pub fn HomePage() -> Element {
    let state = use_context::<Signal<AppState>>();
    let navigator = use_navigator();

    // Redirect to chat if authenticated
    use_effect(move || {
        if state.read().is_authenticated() {
            navigator.push(Route::Chat {});
        }
    });

    rsx! {
        div {
            class: "min-h-screen bg-gradient-to-br from-blue-600 to-purple-700",
            // Navigation
            nav {
                class: "container mx-auto px-6 py-4",
                div {
                    class: "flex items-center justify-between",
                    h1 {
                        class: "text-2xl font-bold text-white",
                        "Rust Teams"
                    }
                    div {
                        class: "space-x-4",
                        Link {
                            to: Route::Login {},
                            class: "text-white hover:text-gray-200 transition-colors",
                            "Sign In"
                        }
                        Link {
                            to: Route::Register {},
                            class: "bg-white text-blue-600 px-4 py-2 rounded-lg font-semibold hover:bg-gray-100 transition-colors",
                            "Get Started"
                        }
                    }
                }
            }

            // Hero section
            div {
                class: "container mx-auto px-6 py-20 text-center",
                h2 {
                    class: "text-5xl font-bold text-white mb-6",
                    "Team Communication,"
                    br {}
                    "Reimagined in Rust"
                }
                p {
                    class: "text-xl text-gray-200 mb-10 max-w-2xl mx-auto",
                    "A blazingly fast, secure, and modern team collaboration platform built entirely in Rust. Chat, call, and share files with your team."
                }
                div {
                    class: "space-x-4",
                    Link {
                        to: Route::Register {},
                        class: "bg-white text-blue-600 px-8 py-3 rounded-lg font-semibold text-lg hover:bg-gray-100 transition-colors inline-block",
                        "Start for Free"
                    }
                    a {
                        href: "https://github.com/jesus-bazan-entel/ApoloTeams",
                        class: "border-2 border-white text-white px-8 py-3 rounded-lg font-semibold text-lg hover:bg-white hover:text-blue-600 transition-colors inline-block",
                        "View on GitHub"
                    }
                }
            }

            // Features section
            div {
                class: "container mx-auto px-6 py-20",
                div {
                    class: "grid md:grid-cols-3 gap-8",
                    // Feature 1
                    div {
                        class: "bg-white bg-opacity-10 rounded-xl p-6 text-white",
                        div {
                            class: "text-4xl mb-4",
                            "💬"
                        }
                        h3 {
                            class: "text-xl font-semibold mb-2",
                            "Real-time Messaging"
                        }
                        p {
                            class: "text-gray-200",
                            "Instant messaging with WebSocket support. Send messages, reactions, and file attachments in real-time."
                        }
                    }
                    // Feature 2
                    div {
                        class: "bg-white bg-opacity-10 rounded-xl p-6 text-white",
                        div {
                            class: "text-4xl mb-4",
                            "📹"
                        }
                        h3 {
                            class: "text-xl font-semibold mb-2",
                            "Video & Audio Calls"
                        }
                        p {
                            class: "text-gray-200",
                            "High-quality video and audio calls powered by WebRTC. Connect with your team face-to-face."
                        }
                    }
                    // Feature 3
                    div {
                        class: "bg-white bg-opacity-10 rounded-xl p-6 text-white",
                        div {
                            class: "text-4xl mb-4",
                            "🔒"
                        }
                        h3 {
                            class: "text-xl font-semibold mb-2",
                            "Secure by Design"
                        }
                        p {
                            class: "text-gray-200",
                            "Built with Rust's memory safety guarantees. Your data is protected with industry-standard encryption."
                        }
                    }
                }
            }

            // Footer
            footer {
                class: "container mx-auto px-6 py-8 text-center text-gray-300",
                p {
                    "Built with ❤️ in Rust • "
                    a {
                        href: "https://github.com/jesus-bazan-entel/ApoloTeams",
                        class: "hover:text-white",
                        "GitHub"
                    }
                }
            }
        }
    }
}
