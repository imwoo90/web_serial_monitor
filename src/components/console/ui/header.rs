use dioxus::prelude::*;

#[component]
pub fn ConsoleHeader(
    autoscroll: bool,
    count: usize,
    onexport: EventHandler<MouseEvent>,
    onclear: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "shrink-0 h-6 bg-[#16181a] border-b border-[#222629] flex items-center justify-between px-3",
            div { class: "flex items-center gap-4",
                div { class: "flex gap-1.5",
                    div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                    div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                    div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                }
                span { class: "text-[10px] text-gray-500 font-mono", "[ LINES: {count} / OPFS ENABLED ]" }
            }
            div { class: "flex items-center gap-2",
                if autoscroll {
                    div { class: "text-[9px] font-mono text-primary/60 uppercase tracking-widest flex items-center gap-1",
                        span { class: "w-1 h-1 rounded-full bg-primary animate-pulse" }
                        "Tracking"
                    }
                } else {
                    div { class: "text-[9px] font-mono text-yellow-500/60 uppercase tracking-widest",
                        "Paused"
                    }
                }

                div { class: "w-px h-3 bg-[#2a2e33]" }

                // Clear Button
                button {
                    class: "flex items-center justify-center w-5 h-5 rounded hover:bg-white/10 transition-colors text-gray-500 hover:text-red-500",
                    onclick: move |evt| onclear.call(evt),
                    title: "Clear Logs",
                    span { class: "material-symbols-outlined text-[14px]", "delete" }
                }

                // Export Button
                button {
                    class: "flex items-center justify-center w-5 h-5 rounded hover:bg-white/10 transition-colors text-gray-500 hover:text-primary",
                    onclick: move |evt| onexport.call(evt),
                    title: "Export Logs",
                    span { class: "material-symbols-outlined text-[14px]", "download" }
                }
            }
        }
    }
}
