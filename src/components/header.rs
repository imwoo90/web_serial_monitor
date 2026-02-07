use crate::components::connection_control::ConnectionControl;
use crate::config::APP_SUBTITLE;
use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    rsx! {
        header { class: "shrink-0 h-14 p-2 flex items-center z-30 relative border-b border-[#2a2e33] bg-[#0d0f10]",
            div { class: "flex gap-3 items-center w-full min-w-[600px]",
                // --- Left: Brand (Aligns with Filter Input: flex-[1.3]) ---
                div { class: "flex-[1.3] flex items-center gap-3 min-w-0 pl-2",
                    div { class: "h-9 w-9 rounded-xl bg-linear-to-br from-primary to-blue-600 flex items-center justify-center shadow-lg shadow-primary/20 shrink-0",
                        span { class: "material-symbols-outlined text-black text-[22px] font-bold",
                            "terminal"
                        }
                    }
                    div { class: "flex flex-col whitespace-nowrap",
                        h1 { class: "text-lg font-bold tracking-tight leading-none text-white",
                            "Serial"
                        }
                        span { class: "text-[10px] font-medium text-gray-400 tracking-wider uppercase",
                            "{APP_SUBTITLE}"
                        }
                    }
                }

                // --- Divider (Matches InputBar Divider) ---
                div { class: "w-px h-8 bg-[#2a2e33]" }

                // --- Right: Controls (Aligns with Send Input: flex-1) ---
                div { class: "flex-1 flex items-center justify-end min-w-0 pr-1", ConnectionControl {} }
            }
        }
    }
}
