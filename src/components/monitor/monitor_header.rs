use crate::components::ui::console::UnifiedConsoleToolbar;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn MonitorHeader(
    autoscroll: bool,
    count: usize,
    onexport: EventHandler<MouseEvent>,
    onclear: EventHandler<MouseEvent>,
    ontoggle_autoscroll: EventHandler<MouseEvent>,
) -> Element {
    let state = use_context::<AppState>();

    rsx! {
        UnifiedConsoleToolbar {
            left: rsx! {
                span { class: "text-[10px] text-gray-500 font-mono", "[ LINES: {count} / OPFS ENABLED ]" }
            },
            font_size: state.ui.font_size,
            is_autoscroll: autoscroll,
            is_tracking_interactive: true,
            on_toggle_autoscroll: move |evt| ontoggle_autoscroll.call(evt),
            on_clear: move |evt| onclear.call(evt),
            on_export: move |evt| onexport.call(evt),
            min_font_size: 8,
            max_font_size: 36,
        }
    }
}
