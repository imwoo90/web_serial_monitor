use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn TerminalToolbar(term_instance: Signal<Option<super::AutoDisposeTerminal>>) -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        div { class: "flex items-center justify-between p-2 bg-gray-800 border-b border-gray-700 text-sm",
            div { class: "flex items-center gap-2",
                // Clear button
                button {
                    class: "px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-white",
                    onclick: move |_| {
                        state.terminal.clear();
                        if let Some(term) = term_instance.read().as_ref() {
                            term.clear();
                        }
                    },
                    "Clear"
                }

                div { class: "w-px h-4 bg-gray-600 mx-1" }

                // Font size controls
                button {
                    class: "px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-white",
                    onclick: move |_| {
                        let current = *state.ui.font_size.read();
                        if current > 8 {
                            *state.ui.font_size.write() = current - 1;
                        }
                    },
                    "-"
                }
                span { class: "text-gray-300", "{state.ui.font_size}px" }
                button {
                    class: "px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-white",
                    onclick: move |_| {
                        let current = *state.ui.font_size.read();
                        if current < 36 {
                            *state.ui.font_size.write() = current + 1;
                        }
                    },
                    "+"
                }
            }

            div { class: "flex items-center gap-2",
                // History config
                span { class: "text-gray-400 text-xs", "History:" }
                input {
                    class: "w-20 px-1 bg-gray-900 border border-gray-700 rounded text-white text-right",
                    r#type: "number",
                    value: "{state.terminal.scrollback}",
                    oninput: move |e| {
                        if let Ok(val) = e.value().parse::<u32>() {
                            *state.terminal.scrollback.write() = val;
                        }
                    },
                }
            }
        }
    }
}
