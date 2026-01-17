use crate::state::{AppState, LineEnding};
use dioxus::prelude::*;

use super::console::{Console, FilterBar, InputBar};
use super::footer::Footer;
use super::header::Header;

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

    // Serial settings state
    let baud_rate = use_signal(|| "115200");
    let data_bits = use_signal(|| "8");
    let stop_bits = use_signal(|| "1");
    let parity = use_signal(|| "None");
    let flow_control = use_signal(|| "None");

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
        baud_rate,
        data_bits,
        stop_bits,
        parity,
        flow_control,
    });

    rsx! {
        div { class: "bg-background-light dark:bg-background-dark h-screen w-full flex flex-col font-display text-white overflow-hidden selection:bg-primary/30 selection:text-primary",

            div { class: "relative shrink-0 z-30",
                Header {}

            }
            FilterBar {}
            Console {}
            InputBar {}
            Footer {}
        }
    }
}
