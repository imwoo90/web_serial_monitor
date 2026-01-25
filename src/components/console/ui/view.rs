use crate::components::console::hooks::bridge::use_worker_bridge;
use crate::components::console::hooks::effects::{use_search_sync, use_settings_sync};
use crate::components::console::ui::viewport::LogViewport;
use crate::state::AppState;
use crate::utils::calculate_window_size;
use dioxus::prelude::*;
use std::rc::Rc;

use crate::components::console::hooks::data_request::use_data_request;
use crate::components::console::ui::header::ConsoleHeader;
use crate::components::console::utils::constants::{
    BOTTOM_BUFFER_EXTRA, CONSOLE_BOTTOM_PADDING, CONSOLE_TOP_PADDING, LINE_HEIGHT, TOP_BUFFER,
};
use crate::components::console::utils::layout_utils::{
    calculate_scroll_state, use_auto_scroller, use_window_resize,
};

#[component]
pub fn Console() -> Element {
    let state = use_context::<AppState>();
    let bridge = use_worker_bridge();

    let mut start_index = use_signal(|| 0usize);
    let mut console_height = use_signal(|| 600.0);
    let total_lines = state.log.total_lines;

    let window_size = calculate_window_size(
        console_height(),
        LINE_HEIGHT,
        TOP_BUFFER + BOTTOM_BUFFER_EXTRA,
    );

    let mut console_handle = use_signal(|| None::<Rc<MountedData>>);
    let mut sentinel_handle = use_signal(|| None::<Rc<MountedData>>);

    // Initial log sync and effects
    use_settings_sync(bridge);
    use_search_sync(bridge);

    // Reset virtual scroll state when logs are cleared or filtered out of view
    use_effect(move || {
        let total = total_lines();
        let start = start_index();

        if total == 0 {
            if start != 0 {
                start_index.set(0);
            }
            return;
        }

        if start >= total {
            if (state.ui.autoscroll)() {
                let page_size = (console_height() / LINE_HEIGHT).ceil() as usize;
                let new_start = total.saturating_sub(page_size);
                if start != new_start {
                    start_index.set(new_start);
                }
            } else if start != 0 {
                start_index.set(0);
                // Reset scroll via JS for non-autoscroll out-of-bounds
                if let Some(el) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id("console-output"))
                {
                    el.set_scroll_top(0);
                }
            }
        }
    });

    use_window_resize(console_height, state.ui.autoscroll, sentinel_handle);
    use_data_request(start_index, window_size, total_lines);
    use_auto_scroller(state.ui.autoscroll, total_lines, sentinel_handle);

    let total_height =
        (total_lines() as f64) * LINE_HEIGHT + CONSOLE_TOP_PADDING + CONSOLE_BOTTOM_PADDING;
    let offset_top = (start_index() as f64) * LINE_HEIGHT;

    rsx! {
        main { class: "flex-1 min-h-0 mx-4 mb-0 mt-0 relative group/console",
            div { class: "absolute inset-0 bg-console-bg rounded-t-2xl border-t border-x border-[#222629] shadow-[inset_0_0_20px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col",
                div { class: "absolute inset-0 scanlines opacity-20 pointer-events-none z-10" }

                ConsoleHeader {
                    autoscroll: (state.ui.autoscroll)(),
                    count: total_lines(),
                    onexport: move |_| bridge.export((state.ui.show_timestamps)()),
                    onclear: move |_| {
                        bridge.clear();
                        state.clear_logs();
                        start_index.set(0);
                        state.success("Logs Cleared");
                    },
                    ontoggle_autoscroll: move |_| state.ui.toggle_autoscroll(),
                }

                LogViewport {
                    total_height,
                    offset_top,
                    onmounted_console: move |evt: MountedEvent| {
                        let handle = evt.data();
                        let h_clone = handle.clone();
                        spawn(async move {
                            if let Ok(rect) = h_clone.get_client_rect().await {
                                console_height.set(rect.height());
                            }
                        });
                        console_handle.set(Some(handle));
                    },
                    onscroll: move |_: ScrollEvent| {
                        let handle = console_handle.peek().as_ref().cloned();
                        spawn(async move {
                            if let Some(handle) = handle {
                                if let Ok(offset) = handle.get_scroll_offset().await {
                                    let (new_index, is_at_bottom) = calculate_scroll_state(
                                        offset.y,
                                        console_height(),
                                        total_lines(),
                                    );
                                    if start_index() != new_index {
                                        start_index.set(new_index);
                                    }
                                    if (state.ui.autoscroll)() != is_at_bottom {
                                        state.ui.set_autoscroll(is_at_bottom);
                                    }
                                }
                            }
                        });
                    },
                    onmounted_sentinel: move |evt: MountedEvent| sentinel_handle.set(Some(evt.data())),
                }

                if !(state.ui.autoscroll)() {
                    ResumeScrollButton { onclick: move |_| state.ui.set_autoscroll(true) }
                }
            }
        }
    }
}

#[component]
pub fn ResumeScrollButton(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "absolute bottom-6 right-6 bg-primary text-surface rounded-full w-10 h-10 shadow-lg shadow-black/50 hover:bg-white active:scale-95 transition-all duration-300 z-20 flex items-center justify-center cursor-pointer group/fab",
            onclick: move |evt| onclick.call(evt),
            span { class: "material-symbols-outlined text-[20px] font-bold", "arrow_downward" }
            span { class: "absolute -top-8 right-0 bg-surface text-[9px] font-bold text-gray-300 px-2 py-1 rounded border border-white/5 opacity-0 group-hover/fab:opacity-100 transition-opacity whitespace-nowrap pointer-events-none uppercase tracking-widest",
                "Resume Scroll"
            }
        }
    }
}
