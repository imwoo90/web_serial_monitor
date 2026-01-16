use super::serial_monitor::AppState;
use dioxus::prelude::*;

#[component]
pub fn FilterSection() -> Element {
    let mut state = use_context::<AppState>();
    let show_highlights = (state.show_highlights)();
    let show_timestamps = (state.show_timestamps)();
    let autoscroll = (state.autoscroll)();

    rsx! {
        div {
            class: "shrink-0 px-5 py-3 z-10 flex flex-col gap-3 filter-section",
            div { class: "flex gap-2 w-full items-stretch",
                div { class: "relative flex-1 group",
                    span { class: "material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-gray-600 text-[20px] group-focus-within:text-primary transition-colors",
                        "search"
                    }
                    input {
                        class: "w-full bg-[#0d0f10] text-sm font-medium text-white placeholder-gray-600 pl-10 pr-30 py-2.5 rounded-xl border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none shadow-inset-input transition-all",
                        placeholder: "Filter output...",
                        "type": "text"
                    }
                    div { class: "absolute right-1.5 top-1/2 -translate-y-1/2 flex items-center gap-1",
                        button {
                            class: "w-8 h-7 flex items-center justify-center rounded-md hover:bg-[#2a2e33] text-gray-500 hover:text-white transition-all focus:outline-none",
                            title: "Match Case",
                            "aria-label": "Match Case",
                            span { class: "text-[11px] font-bold font-mono", "Aa" }
                        }
                        button {
                            class: "w-8 h-7 flex items-center justify-center rounded-md bg-primary/10 border border-primary/20 text-primary shadow-[0_0_10px_rgba(0,191,255,0.15)] transition-all focus:outline-none",
                            title: "Use Regex",
                            "aria-label": "Regex Mode",
                            span { class: "text-[11px] font-bold font-mono", ".*" }
                        }
                        button {
                            class: "w-8 h-7 flex items-center justify-center rounded-md hover:bg-[#2a2e33] text-gray-500 hover:text-white transition-all focus:outline-none",
                            title: "Invert Filter",
                            "aria-label": "Invert Filter",
                            span { class: "text-[13px] font-bold font-mono", "!" }
                        }
                    }
                }
                button {
                    class: "highlight-icon-btn w-12 flex items-center justify-center rounded-xl border transition-all active:scale-95 shadow-inset-input",
                    class: if show_highlights { "text-primary bg-primary/10 border-primary/50" } else { "bg-[#0d0f10] border-[#2a2e33] text-gray-500 hover:text-white hover:border-primary/50" },
                    onclick: move |_| {
                        let current = (state.show_highlights)();
                        state.show_highlights.set(!current);
                    },
                    title: "Highlighter",
                    span { class: "material-symbols-outlined text-[20px]", "ink_highlighter" }
                }
            }
            div {
                class: "highlight-panel overflow-hidden transition-all duration-300 bg-surface rounded-xl border border-white/10 shadow-lg",
                class: if show_highlights { "max-h-[400px] opacity-100 visible p-4 mt-2" } else { "max-h-0 opacity-0 invisible" },
                div { class: "flex flex-col gap-3",
                    div { class: "flex items-center justify-between border-b border-white/5 pb-2",
                        span { class: "text-[11px] font-bold text-gray-500 uppercase tracking-widest", "Active Highlights" }
                        span { class: "text-[10px] text-gray-600", "2 active rules" }
                    }
                    div { class: "flex flex-wrap gap-2",
                        div { class: "flex items-center gap-2 pl-3 pr-2 py-1.5 bg-[#0d0f10] border border-red-500/30 rounded-full group hover:border-red-500/60 transition-colors",
                            div { class: "w-2 h-2 rounded-full bg-red-500 shadow-[0_0_5px_rgba(239,68,68,0.5)]" }
                            span { class: "text-xs font-bold text-gray-300", "Warning" }
                            button { class: "ml-1 hover:text-white text-gray-500 rounded-full w-4 h-4 flex items-center justify-center transition-colors",
                                span { class: "material-symbols-outlined text-[14px]", "close" }
                            }
                        }
                        div { class: "flex items-center gap-2 pl-3 pr-2 py-1.5 bg-[#0d0f10] border border-blue-500/30 rounded-full group hover:border-blue-500/60 transition-colors",
                            div { class: "w-2 h-2 rounded-full bg-blue-500 shadow-[0_0_5px_rgba(59,130,246,0.5)]" }
                            span { class: "text-xs font-bold text-gray-300", "RX" }
                            button { class: "ml-1 hover:text-white text-gray-500 rounded-full w-4 h-4 flex items-center justify-center transition-colors",
                                span { class: "material-symbols-outlined text-[14px]", "close" }
                            }
                        }
                    }
                    div { class: "pt-2 border-t border-white/5 flex gap-2",
                        input {
                            class: "flex-1 bg-[#0d0f10] text-xs font-medium text-white placeholder-gray-600 px-3 py-2 rounded-lg border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none transition-all",
                            placeholder: "New keyword...",
                            "type": "text"
                        }
                        div { class: "flex gap-1 bg-[#0d0f10] p-1 rounded-lg border border-[#2a2e33]",
                            button { class: "w-6 h-full rounded bg-yellow-500 hover:scale-110 transition-transform" }
                            button { class: "w-6 h-full rounded bg-green-500 hover:scale-110 transition-transform" }
                            button { class: "w-6 h-full rounded bg-purple-500 hover:scale-110 transition-transform" }
                            button { class: "w-6 h-full rounded bg-primary hover:scale-110 transition-transform" }
                        }
                        button { class: "px-3 rounded-lg bg-white/5 hover:bg-white/10 text-white border border-white/10 transition-colors",
                            span { class: "material-symbols-outlined text-[18px]", "add" }
                        }
                    }
                }
            }
            div { class: "flex items-center gap-6",
                button {
                    class: "flex items-center cursor-pointer group gap-2",
                    onclick: move |_| {
                        let current = (state.show_timestamps)();
                        state.show_timestamps.set(!current);
                    },
                    div { class: "relative flex items-center",
                        div {
                            class: "w-7 h-3.5 rounded-full transition-all duration-200 border border-white/5",
                            class: if show_timestamps { "bg-primary border-primary shadow-[0_0_8px_rgba(0,191,255,0.4)]" } else { "bg-[#2a2e33] group-hover:bg-[#34393e]" }
                        }
                        div {
                            class: "absolute left-0 w-3.5 h-3.5 rounded-full transition-all duration-200",
                            class: if show_timestamps { "translate-x-3.5 bg-white" } else { "bg-gray-500" }
                        }
                    }
                    span {
                        class: "text-[10px] font-bold uppercase tracking-widest transition-colors",
                        class: if show_timestamps { "text-primary" } else { "text-gray-500 group-hover:text-gray-300" },
                        "Timestamp"
                    }
                }
                button {
                    class: "flex items-center cursor-pointer group gap-2",
                    onclick: move |_| {
                        let current = (state.autoscroll)();
                        state.autoscroll.set(!current);
                    },
                    div { class: "relative flex items-center",
                        div {
                            class: "w-7 h-3.5 rounded-full transition-all duration-200 border border-white/5",
                            class: if autoscroll { "bg-primary border-primary shadow-[0_0_8px_rgba(0,191,255,0.4)]" } else { "bg-[#2a2e33] group-hover:bg-[#34393e]" }
                        }
                        div {
                            class: "absolute left-0 w-3.5 h-3.5 rounded-full transition-all duration-200",
                            class: if autoscroll { "translate-x-3.5 bg-white" } else { "bg-gray-500" }
                        }
                    }
                    span {
                        class: "text-[10px] font-bold uppercase tracking-widest transition-colors leading-none",
                        class: if autoscroll { "text-primary" } else { "text-gray-500 group-hover:text-gray-300" },
                        "Auto-scroll"
                    }
                }
                div { class: "ml-auto text-[10px] font-bold text-gray-500 uppercase tracking-widest flex items-center gap-2",
                    span { class: "w-1.5 h-1.5 rounded-full bg-primary animate-pulse" }
                    "Live"
                }
            }
        }
    }
}
