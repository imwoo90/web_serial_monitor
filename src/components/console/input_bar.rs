use crate::components::console::{SearchBar, TransmitBar};
use dioxus::prelude::*;

#[component]
pub fn InputBar() -> Element {
    rsx! {
        div { class: "shrink-0 p-2 bg-background-dark z-20 relative",
            div { class: "flex gap-3 h-10 items-stretch min-w-[600px]",
                SearchBar {}
                // --- Divider ---
                div { class: "w-px bg-[#2a2e33] my-1" }
                TransmitBar {}
            }
        }
    }
}
