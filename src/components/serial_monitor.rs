use crate::components::common::ToastContainer;
use crate::state::{AppState, LineEnding};
use dioxus::prelude::*;

use super::console::{Console, FilterBar, InputBar, MacroBar};
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
    let mut log_worker = use_signal(|| None::<web_sys::Worker>);
    let toasts = use_signal(Vec::new);

    use_effect(move || {
        if log_worker.peek().is_none() {
            let worker_path = asset!("/assets/js/log_worker.js").to_string();
            let opts = web_sys::WorkerOptions::new();
            opts.set_type(web_sys::WorkerType::Module);
            if let Ok(w) = web_sys::Worker::new_with_options(&worker_path, &opts) {
                log_worker.set(Some(w));
            }
        }
    });

    // Sync RX Line Ending to Worker
    use_effect(move || {
        let ending = rx_line_ending();
        if let Some(w) = log_worker.read().as_ref() {
            let mode_str = match ending {
                LineEnding::None => "None",
                LineEnding::NL => "NL",
                LineEnding::CR => "CR",
                LineEnding::NLCR => "NLCR",
            };
            crate::utils::send_worker_msg(
                w,
                crate::components::console::types::WorkerMsg::SetLineEnding(mode_str.to_string()),
            );
        }
    });

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
        rx_line_ending,
        is_hex_view,
        tx_local_echo,
        port,
        reader,
        is_connected,
        is_simulating,
        log_worker,
        toasts,
    });

    rsx! {
        div { class: "bg-background-light dark:bg-background-dark h-screen w-full font-display text-white selection:bg-primary/30 selection:text-primary overflow-x-auto overflow-y-hidden",
            div { class: "flex flex-col h-full min-w-[600px]",
                div { class: "relative shrink-0 z-30", Header {} }
                InputBar {}
                FilterBar {}
                Console {}
                MacroBar {}
                ToastContainer { toasts }
            }
        }
    }
}
