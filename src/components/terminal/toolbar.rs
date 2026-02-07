use crate::components::ui::console::UnifiedConsoleToolbar;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn TerminalToolbar(term_instance: Signal<Option<super::AutoDisposeTerminal>>) -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        UnifiedConsoleToolbar {
            left: rsx! {
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
            },
            font_size: state.ui.font_size,
            is_autoscroll: *state.terminal.autoscroll.read(),
            is_tracking_interactive: false,
            on_toggle_autoscroll: |_| {}, // No-op
            on_clear: move |_| {
                state.terminal.clear();
                if let Some(term) = term_instance.read().as_ref() {
                    term.clear();
                }
            },
            on_export: None,
            min_font_size: 8,
            max_font_size: 36,
        }
    }
}
