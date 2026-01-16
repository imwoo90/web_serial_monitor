use crate::state::{AppState, Highlight, LineEnding};
use dioxus::prelude::*;

use super::console::Console;
use super::filter_section::FilterSection;
use super::footer::Footer;
use super::header::Header;
use super::settings_panel::SettingsPanel;

#[component]
pub fn SerialMonitor() -> Element {
    // Initialize common state
    let show_settings = use_signal(|| false);
    let show_highlights = use_signal(|| false);
    let show_timestamps = use_signal(|| true);
    let autoscroll = use_signal(|| true);
    let line_ending = use_signal(|| LineEnding::None);
    let highlights = use_signal(Vec::new);
    let filter_query = use_signal(|| String::new());
    let match_case = use_signal(|| false);
    let use_regex = use_signal(|| false);
    let invert_filter = use_signal(|| false);

    use_context_provider(|| AppState {
        show_settings,
        show_highlights,
        show_timestamps,
        autoscroll,
        line_ending,
        highlights,
        filter_query,
        match_case,
        use_regex,
        invert_filter,
    });

    rsx! {
        div { class: "bg-background-light dark:bg-background-dark h-screen w-full flex flex-col font-display text-white overflow-hidden selection:bg-primary/30 selection:text-primary",

            div { class: "relative shrink-0 z-30",
                Header {}
                SettingsPanel {}
            }
            FilterSection {}
            Console {}
            Footer {}
        }
    }
}
