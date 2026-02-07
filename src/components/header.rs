use crate::components::connection_control::ConnectionControl;
use crate::config::APP_SUBTITLE;
use crate::state::{AppState, ViewMode};
use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    let app = use_context::<AppState>();
    let view_mode = app.ui.view_mode;
    let mut show_menu = use_signal(|| false);

    // Close menu when clicking backdrop
    let close_menu = move |_| show_menu.set(false);

    rsx! {
        header { class: "shrink-0 h-14 p-2 flex items-center z-30 relative border-b border-[#2a2e33] bg-[#0d0f10]",
            div { class: "flex gap-3 items-center w-full min-w-[600px]",
                // --- Left: Brand & Mode Switcher ---
                div { class: "flex-[1.3] relative min-w-0",
                    // Trigger Button
                    button {
                        class: "flex items-center gap-3 pl-2 pr-4 py-1.5 -ml-2 rounded-xl transition-all duration-200 hover:bg-[#1e2024] group outline-none",
                        onclick: move |_| show_menu.toggle(),
                        div { class: format!("h-9 w-9 rounded-xl flex items-center justify-center shadow-lg transition-colors {}",
                                if view_mode() == ViewMode::Terminal { "bg-gray-700 shadow-gray-900/20" } else { "bg-linear-to-br from-primary to-blue-600 shadow-primary/20" }
                            ),
                            span { class: "material-symbols-outlined text-white text-[20px]",
                                if view_mode() == ViewMode::Terminal { "terminal" } else { "list_alt" }
                            }
                        }
                        div { class: "flex flex-col items-start whitespace-nowrap",
                            div { class: "flex items-center gap-1.5",
                                h1 { class: "text-lg font-bold tracking-tight leading-none text-white group-hover:text-primary transition-colors",
                                    if view_mode() == ViewMode::Terminal { "Terminal" } else { "Monitor" }
                                }
                                span { class: "material-symbols-outlined text-gray-500 text-[16px] transition-transform duration-200 group-hover:text-gray-300",
                                    if show_menu() { "expand_less" } else { "expand_more" }
                                }
                            }
                            span { class: "text-[10px] font-medium text-gray-400 tracking-wider uppercase",
                                "{APP_SUBTITLE}"
                            }
                        }
                    }

                    // Dropdown Menu
                    if show_menu() {
                        div {
                            class: "absolute top-full left-0 mt-2 w-48 bg-[#1e2024] border border-[#2a2e33] rounded-xl shadow-2xl overflow-hidden z-50 flex flex-col py-1 animation-files-enter",
                            // Backdrop to close
                            div { class: "fixed inset-0 z-[-1]", onclick: close_menu }

                            // Monitor Option
                            button {
                                class: "flex items-center gap-3 px-4 py-2.5 text-left hover:bg-[#2a2e33] transition-colors group",
                                onclick: move |_| {
                                    app.ui.set_view_mode(ViewMode::Monitoring);
                                    show_menu.set(false);
                                },
                                span { class: format!("material-symbols-outlined text-[20px] {}",
                                    if view_mode() == ViewMode::Monitoring { "text-primary" } else { "text-gray-500 group-hover:text-gray-300" }
                                ), "list_alt" }
                                div { class: "flex flex-col",
                                    span { class: format!("text-sm font-medium leading-none mb-0.5 {}",
                                        if view_mode() == ViewMode::Monitoring { "text-white" } else { "text-gray-300 group-hover:text-white" }
                                    ), "Monitor" }
                                    span { class: "text-[10px] text-gray-500 group-hover:text-gray-400", "Log Viewer & Analysis" }
                                }
                            }

                            // Terminal Option
                            button {
                                class: "flex items-center gap-3 px-4 py-2.5 text-left hover:bg-[#2a2e33] transition-colors group border-t border-[#2a2e33]",
                                onclick: move |_| {
                                    app.ui.set_view_mode(ViewMode::Terminal);
                                    show_menu.set(false);
                                },
                                span { class: format!("material-symbols-outlined text-[20px] {}",
                                    if view_mode() == ViewMode::Terminal { "text-primary" } else { "text-gray-500 group-hover:text-gray-300" }
                                ), "terminal" }
                                div { class: "flex flex-col",
                                    span { class: format!("text-sm font-medium leading-none mb-0.5 {}",
                                        if view_mode() == ViewMode::Terminal { "text-white" } else { "text-gray-300 group-hover:text-white" }
                                    ), "Terminal" }
                                    span { class: "text-[10px] text-gray-500 group-hover:text-gray-400", "Details for v3.0.0" }
                                }
                            }
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
