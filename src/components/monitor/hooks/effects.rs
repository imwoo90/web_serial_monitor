use crate::hooks::WorkerController;
use crate::state::AppState;
use dioxus::prelude::*;

pub fn use_settings_sync(bridge: WorkerController) {
    let state = use_context::<AppState>();

    use_effect(move || {
        let show = (state.ui.show_timestamps)();
        bridge.set_timestamp_state(show);
    });
}

pub fn use_search_sync(bridge: WorkerController) {
    let state = use_context::<AppState>();

    use_resource(move || {
        let query = (state.log.filter_query)();
        let match_case = (state.log.match_case)();
        let use_regex = (state.log.use_regex)();
        let invert = (state.log.invert_filter)();

        async move {
            // Debounce 300ms
            gloo_timers::future::TimeoutFuture::new(300).await;
            bridge.search(query, match_case, use_regex, invert);
        }
    });
}
