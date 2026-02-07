use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn TerminalToolbar(term_instance: Signal<Option<super::AutoDisposeTerminal>>) -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        div { class: "shrink-0 h-6 bg-[#16181a] border-b border-[#222629] flex items-center justify-between px-3",
            div { class: "flex items-center gap-4",
                div { class: "w-px h-3 bg-[#2a2e33]" }

                // History Config & Line Count
                div { class: "flex items-center gap-2",
                    span { class: "text-[10px] text-gray-500 font-mono", "[ LINES: {state.terminal.lines} / HISTORY:" }
                    input {
                        class: "w-12 h-4 px-1 bg-[#0b0c0d] border border-[#222629] rounded text-[10px] text-gray-300 text-right focus:border-primary focus:outline-none transition-colors",
                        r#type: "number",
                        value: "{state.terminal.scrollback}",
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u32>() {
                                *state.terminal.scrollback.write() = val;
                            }
                        },
                    }
                    span { class: "text-[10px] text-gray-500 font-mono", "]" }
                }
            }

            div { class: "flex items-center gap-2",
                // Font Size Controls
                div { class: "flex items-center gap-1",
                    button {
                        class: "flex items-center justify-center w-5 h-5 rounded hover:bg-white/10 transition-colors text-gray-500 hover:text-white",
                        onclick: move |_| {
                            let current = *state.ui.font_size.read();
                            if current > 8 {
                                *state.ui.font_size.write() = current - 1;
                            }
                        },
                        title: "Decrease Font Size",
                        span { class: "text-[10px] font-bold", "-" }
                    }
                    span { class: "text-[10px] text-gray-500 font-mono w-6 text-center", "{state.ui.font_size}px" }
                    button {
                        class: "flex items-center justify-center w-5 h-5 rounded hover:bg-white/10 transition-colors text-gray-500 hover:text-white",
                        onclick: move |_| {
                            let current = *state.ui.font_size.read();
                            if current < 36 {
                                *state.ui.font_size.write() = current + 1;
                            }
                        },
                        title: "Increase Font Size",
                        span { class: "text-[10px] font-bold", "+" }
                    }
                }

                div { class: "w-px h-3 bg-[#2a2e33]" }

                // Auto-scroll Indicator
                div {
                    class: "cursor-pointer group/tracking select-none",
                    onclick: move |_| {
                        // Toggle autoscroll
                         let new_state = !*state.terminal.autoscroll.read();
                         *state.terminal.autoscroll.write() = new_state;

                         // If enabled, scroll to bottom immediately
                         if new_state {
                             if let Some(term) = term_instance.read().as_ref() {
                                 term.scroll_to_bottom();
                             }
                         }
                    },
                    if *state.terminal.autoscroll.read() {
                        div { class: "text-[9px] font-mono text-primary/60 uppercase tracking-widest flex items-center gap-1 group-hover/tracking:text-primary transition-colors",
                            span { class: "w-1 h-1 rounded-full bg-primary animate-pulse" }
                            "Tracking"
                        }
                    } else {
                        div { class: "text-[9px] font-mono text-yellow-500/60 uppercase tracking-widest group-hover/tracking:text-yellow-500 transition-colors",
                            "Paused"
                        }
                    }
                }

                div { class: "w-px h-3 bg-[#2a2e33]" }

                // Clear Button
                button {
                    class: "flex items-center justify-center w-5 h-5 rounded hover:bg-white/10 transition-colors text-gray-500 hover:text-red-500",
                    onclick: move |_| {
                        state.terminal.clear();
                        if let Some(term) = term_instance.read().as_ref() {
                            term.clear();
                        }
                    },
                    title: "Clear Terminal",
                    span { class: "material-symbols-outlined text-[14px]", "delete" }
                }
            }
        }
    }
}
