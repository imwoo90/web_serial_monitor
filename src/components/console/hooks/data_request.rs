use dioxus::prelude::*;

/// Hook to request a window of log data from Worker
pub fn use_data_request(
    start_index: Signal<usize>,
    window_size: usize,
    total_lines: Signal<usize>,
) {
    let bridge = crate::hooks::use_worker_controller();
    let mut prev_start = use_signal(|| 0usize);
    use_effect(move || {
        let start = start_index();
        let total = total_lines();
        let is_moved = start != *prev_start.peek();

        if is_moved || total < window_size {
            bridge.request_window(start, window_size);
            prev_start.set(start);
        }
    });
}
