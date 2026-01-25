use crate::components::common::ToastContainer;
use crate::state::{AppState, LineEnding};
use dioxus::prelude::*;

use super::console::{Console, FilterBar, InputBar, MacroBar};
use crate::components::header::Header;
use crate::hooks::use_log_worker;

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
    let baud_rate = use_signal(|| "115200".to_string());
    let data_bits = use_signal(|| "8");
    let stop_bits = use_signal(|| "1");
    let parity = use_signal(|| "None");
    let flow_control = use_signal(|| "None");
    let rx_line_ending = use_signal(|| LineEnding::NL);
    let is_hex_view = use_signal(|| false);
    let tx_local_echo = use_signal(|| false);
    let port = use_signal(|| None);
    let reader = use_signal(|| None);
    let is_connected = use_signal(|| false);
    let is_simulating = use_signal(|| false);
    let log_worker = use_signal(|| None::<web_sys::Worker>);
    let toasts = use_signal(Vec::new);
    let total_lines = use_signal(|| 0usize);
    let visible_logs = use_signal(|| Vec::<String>::new());

    let app_state = AppState {
        ui: crate::state::UIState {
            show_settings,
            show_highlights,
            show_timestamps,
            autoscroll,
            is_hex_view,
        },
        serial: crate::state::SerialSettings {
            baud_rate,
            data_bits,
            stop_bits,
            parity,
            flow_control,
            rx_line_ending,
            tx_line_ending: line_ending,
            tx_local_echo,
        },
        conn: crate::state::ConnectionState {
            port,
            reader,
            is_connected,
            is_simulating,
            log_worker,
        },
        log: crate::state::LogState {
            total_lines,
            visible_logs,
            filter_query,
            match_case,
            use_regex,
            invert_filter,
            highlights,
            toasts,
        },
    };

    use_context_provider(|| app_state);

    // Lifecycle/Effects Hook
    use_log_worker(app_state);

    rsx! {
        div { class: "bg-background-dark h-screen w-full font-display text-white selection:bg-primary/30 selection:text-primary overflow-x-auto overflow-y-hidden",
            div { class: "flex flex-col h-full min-w-[600px]",
                Header {}
                InputBar {}
                FilterBar {}
                Console {}
                MacroBar {}
                ToastContainer { toasts }
            }
        }
    }
}
