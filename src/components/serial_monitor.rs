use dioxus::prelude::*;

use super::console::Console;
use super::filter_section::FilterSection;
use super::footer::Footer;
use super::header::Header;
use super::settings_panel::SettingsPanel;

#[derive(Clone, Copy, PartialEq)]
pub enum LineEnding {
    None,
    NL,
    CR,
    NLCR,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub show_settings: Signal<bool>,
    pub show_highlights: Signal<bool>,
    pub show_timestamps: Signal<bool>,
    pub autoscroll: Signal<bool>,
    pub line_ending: Signal<LineEnding>,
}

#[component]
pub fn SerialMonitor() -> Element {
    // Initialize common state
    let show_settings = use_signal(|| false);
    let show_highlights = use_signal(|| false);
    let show_timestamps = use_signal(|| true);
    let autoscroll = use_signal(|| true);
    let line_ending = use_signal(|| LineEnding::None);

    use_context_provider(|| AppState {
        show_settings,
        show_highlights,
        show_timestamps,
        autoscroll,
        line_ending,
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
