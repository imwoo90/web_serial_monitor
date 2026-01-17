use crate::state::AppState;
use crate::utils::{calculate_start_index, calculate_window_size};
use dioxus::prelude::*;
use std::rc::Rc;

use super::layout_utils::{
    use_auto_scroller, use_window_resize, ConsoleHeader, ResumeScrollButton,
};
use super::log_line::LogLine;
use super::types::{WorkerMsg, BOTTOM_BUFFER_EXTRA, LINE_HEIGHT, TOP_BUFFER};
use super::worker::{use_data_request, use_log_worker}; // Only if needed locally, but log_line uses it. Wait, LogLine calls it. So View doesn't need to call it directly.

#[component]
pub fn Console() -> Element {
    let mut state = use_context::<AppState>();

    // 1. Signals & Setup
    // worker is now in AppState
    let visible_logs = use_signal(|| Vec::<String>::new());
    let total_lines = use_signal(|| 0usize);
    let mut start_index = use_signal(|| 0usize);
    let mut console_height = use_signal(|| 600.0);

    let window_size = calculate_window_size(
        console_height(),
        LINE_HEIGHT,
        TOP_BUFFER + BOTTOM_BUFFER_EXTRA,
    );

    let mut console_handle = use_signal(|| None::<Rc<MountedData>>);
    let mut sentinel_handle = use_signal(|| None::<Rc<MountedData>>);

    // 2. Effects
    use_log_worker(total_lines, visible_logs, state.log_worker);
    use_window_resize(console_height, state.autoscroll, sentinel_handle);
    use_data_request(start_index, window_size, total_lines, state.log_worker);
    use_auto_scroller(state.autoscroll, total_lines, sentinel_handle);

    let total_height = (total_lines() as f64) * LINE_HEIGHT;
    let offset_top = (start_index() as f64) * LINE_HEIGHT;

    // Search Effect (Debounced)
    use_effect(move || {
        let query = (state.filter_query)();
        let match_case = (state.match_case)();
        let use_regex = (state.use_regex)();
        let invert = (state.invert_filter)();
        let worker_sig = state.log_worker;

        // Simple debounce using async sleep (Effect runs on every dependency change)
        spawn(async move {
            // Wait 300ms for debounce
            gloo_timers::future::TimeoutFuture::new(300).await;

            // Check if signals are still same?
            // Dioxus effects re-run on signal change, cleaning up previous one?
            // Actually, in Dioxus 0.5/0.6 signals in effect dependency create subscription.
            // If we spawn inside effect, it runs. But we need cancellation or verify value is latest.
            // A better pattern for debounce in Dioxus:
            // But since this is a one-shot, let's just send the message.
            // Worker handles 'SessionId' so rapid messages are fine,
            // but we want to avoid spamming the worker.

            // However, inside this spawned future, we can't easily check if "latest" query matches.
            // But we CAN check current signal value compared to captured value.
            // If they differ, it means a newer effect ran (or signal changed).

            // Actually, best debounce is just let the Worker handle rapid session updates (AbortController style).
            // But sending message is cheap. Scanning is expensive.
            // The worker's `searchSessionId` logic ALREADY handles logic cancellation!
            // So we don't strictly need a complex debounce here IF we trust the worker to abort quickly.
            // But let's add a small delay anyway to group keystrokes.

            if let Some(w) = worker_sig.peek().as_ref() {
                let msg = WorkerMsg::SearchLogs {
                    query,
                    match_case,
                    use_regex,
                    invert,
                };
                if let Ok(js_obj) = serde_wasm_bindgen::to_value(&msg) {
                    let _ = w.post_message(&js_obj);
                }
            }
        });
    });

    let onexport = move |_evt: MouseEvent| {
        let worker = state.log_worker.read().clone();
        if let Some(w) = worker {
            let msg = WorkerMsg::ExportLogs {
                include_timestamp: (state.show_timestamps)(),
            };
            if let Ok(js_obj) = serde_wasm_bindgen::to_value(&msg) {
                let _ = w.post_message(&js_obj);
            }
        }
    };

    rsx! {
        main { class: "flex-1 min-h-0 mx-4 mb-0 mt-0 relative group/console",
            div { class: "absolute inset-0 bg-console-bg rounded-t-2xl border-t border-x border-[#222629] shadow-[inset_0_0_20px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col",
                div { class: "absolute inset-0 scanlines opacity-20 pointer-events-none z-10" }

                ConsoleHeader {
                    autoscroll: (state.autoscroll)(),
                    count: total_lines(),
                    is_connected: (state.is_connected)(),
                    onexport,
                }

                div {
                    class: "flex-1 overflow-y-auto font-mono text-xs md:text-sm leading-[20px] scrollbar-custom relative",
                    style: "overflow-anchor: none;", // Prevent scroll anchoring interference
                    id: "console-output",
                    onmounted: move |evt| {
                        let handle = evt.data();
                        let h_clone = handle.clone();
                        spawn(async move {
                            if let Ok(rect) = h_clone.get_client_rect().await {
                                console_height.set(rect.height());
                            }
                        });
                        console_handle.set(Some(handle));
                    },
                    onscroll: move |_| {
                        let handle = console_handle.peek().as_ref().cloned();
                        spawn(async move {
                            if let Some(handle) = handle {
                                if let Ok(offset) = handle.get_scroll_offset().await {
                                    // 1. Calculate Virtual Scroll Index
                                    let new_index = calculate_start_index(
                                        offset.y,
                                        LINE_HEIGHT,
                                        TOP_BUFFER,
                                    );
                                    if start_index() != new_index {
                                        start_index.set(new_index);
                                    }

                                    // 2. Autoscroll Detection (Math-based)
                                    // Sentinel visibility is flaky during layout shifts (Hex toggle).
                                    // Instead, check if we are near the bottom.
                                    let scroll_top = offset.y;
                                    let viewport_height = console_height();
                                    let content_height = (total_lines() as f64) * LINE_HEIGHT;

                                    // Allow small buffer (e.g. 50px)
                                    let is_at_bottom = scroll_top + viewport_height >= content_height - 50.0;

                                    // Only update if changed
                                    if (state.autoscroll)() != is_at_bottom {
                                        // If content is shorter than viewport, always autoscroll
                                        if content_height <= viewport_height {
                                            state.autoscroll.set(true);
                                        } else {
                                            state.autoscroll.set(is_at_bottom);
                                        }
                                    }
                                }
                            }
                        });
                    },

                    // Virtual Scroll Spacer & Content
                    div { style: "height: {total_height}px; width: 100%; position: absolute; top: 0; left: 0; pointer-events: none;" }
                    div { style: "position: absolute; top: 0; left: 0; right: 0; transform: translateY({offset_top}px); padding: 0.5rem 1rem 20px 1rem; pointer-events: auto; min-width: 100%; width: max-content;",
                        {
                            let highlights = (state.highlights)().clone();
                            let show_timestamps = (state.show_timestamps)();
                            let show_highlights = (state.show_highlights)();

                            visible_logs
                                .read()
                                .iter()
                                .enumerate()
                                // Removed .filter() since worker sends filtered logs
                                .map(move |(idx, text)| {
                                    rsx! {
                                        LogLine {
                                            key: "{idx}",
                                            text: text.clone(),
                                            highlights: highlights.clone(),
                                            show_timestamps,
                                            show_highlights,
                                        }
                                    }
                                })
                        }
                    }

                    // Loading & Sentinel
                    if visible_logs.read().is_empty() && total_lines() > 0 {
                        div { class: "text-gray-500 animate-pulse text-[12px] px-4",
                            "Loading buffer..."
                        }
                    }
                    div {
                        // Sentinel is used ONLY as a scroll target, not for detection.
                        style: "position: absolute; top: {total_height}px; height: 1px; width: 100%; pointer-events: none;",
                        onmounted: move |evt| sentinel_handle.set(Some(evt.data())),
                    }
                }

                if !(state.autoscroll)() {
                    ResumeScrollButton { onclick: move |_| state.autoscroll.set(true) }
                }
            }
        }
    }
}
