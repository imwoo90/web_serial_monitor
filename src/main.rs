use dioxus::prelude::*;

mod components;
mod config;
mod hooks;
mod state;
pub mod types;
mod utils;
mod worker;
use components::serial_monitor::SerialMonitor;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const XTERM_CSS: Asset = asset!("/assets/css/xterm.css");
const XTERM_JS: Asset = asset!("/assets/js/xterm.js");
const XTERM_FIT_ADDON_JS: Asset = asset!("/assets/js/xterm-addon-fit.js");

fn main() {
    #[cfg(target_arch = "wasm32")]
    if crate::worker::lifecycle::start_worker() {
        return;
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Title { "RusTerm - High Performance Serial Monitor & Terminal" }
        document::Script { "document.documentElement.classList.add('dark')" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            crossorigin: "anonymous",
        }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&display=swap",
        }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap",
        }
        document::Link { rel: "stylesheet", href: "{TAILWIND_CSS}" }
        document::Link { rel: "stylesheet", href: "{XTERM_CSS}" }
        document::Script { src: "{XTERM_JS}" }
        document::Script { src: "{XTERM_FIT_ADDON_JS}" }
        SerialMonitor {}
    }
}
