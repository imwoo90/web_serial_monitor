use dioxus::prelude::*;

/// Hook to request a window of log data from Worker
pub fn use_data_request(
    start_index: Signal<usize>,
    window_size: usize,
    total_lines: Signal<usize>,
) {
    let state = use_context::<crate::state::AppState>();
    let bridge = crate::hooks::use_worker_controller();
    let prev_start = use_signal(|| usize::MAX);

    use_effect(move || {
        let start = start_index();
        let total = total_lines();
        // Use peek to avoid subscribing to visible_logs updates, preventing infinite loops
        let visible = state.log.visible_logs.peek().len();
        let mut last_s = prev_start;

        // 1. If start index changed (scroll/nav), we MUST update.
        let start_changed = *last_s.peek() != start;

        // 2. If we haven't filled the window yet, but more lines exist, we MUST update.
        //    (This fixes the startup/"Loading buffer..." issue)
        let needs_more_data = visible < window_size && total > visible;

        if start_changed || needs_more_data {
            bridge.request_window(start, window_size);
            last_s.set(start);
        }
    });
}
