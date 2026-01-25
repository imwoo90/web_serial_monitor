use super::bridge::WorkerBridge;
use crate::state::AppState;
use dioxus::prelude::*;

pub fn use_settings_sync(bridge: WorkerBridge) {
    let state = use_context::<AppState>();

    // RX Line Ending Sync
    use_effect(move || {
        let ending = (state.serial.rx_line_ending)();
        bridge.set_line_ending(format!("{:?}", ending));
    });
}

pub fn use_search_sync(bridge: WorkerBridge) {
    let state = use_context::<AppState>();

    use_effect(move || {
        let query = (state.log.filter_query)();
        let match_case = (state.log.match_case)();
        let use_regex = (state.log.use_regex)();
        let invert = (state.log.invert_filter)();

        spawn(async move {
            // Debounce 300ms
            gloo_timers::future::TimeoutFuture::new(300).await;
            bridge.search(query, match_case, use_regex, invert);
        });
    });
}
