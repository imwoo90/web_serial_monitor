use dioxus::prelude::*;

/// Hook to request a window of log data from Worker
pub fn use_data_request(
    start_index: Signal<usize>,
    window_size: usize,
    _total_lines: Signal<usize>,
) {
    let bridge = crate::hooks::use_worker_bridge();
    use_effect(move || {
        let start = start_index();
        bridge.request_window(start, window_size);
    });
}
