use crate::components::console::hooks::effects::{use_search_sync, use_settings_sync};
use crate::components::console::viewport::LogViewport;
use crate::hooks::use_worker_controller;
use crate::state::AppState;
use dioxus::prelude::*;

use crate::components::console::console_header::ConsoleHeader;
use crate::components::console::hooks::virtual_scroll::use_virtual_scroll;
use crate::components::console::utils::layout_utils::calculate_scroll_state;

#[component]
pub fn Console() -> Element {
    let state = use_context::<AppState>();
    let bridge = use_worker_controller();
    let mut vs = use_virtual_scroll();

    // Initial log sync and effects
    use_settings_sync(bridge);
    use_search_sync(bridge);

    rsx! {
        main { class: "flex-1 min-h-0 mx-4 mb-0 mt-0 relative group/console",
            div { class: "absolute inset-0 bg-console-bg rounded-t-2xl border-t border-x border-[#222629] shadow-[inset_0_0_20px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col",
                div { class: "absolute inset-0 scanlines opacity-20 pointer-events-none z-10" }

                ConsoleHeader {
                    autoscroll: (state.ui.autoscroll)(),
                    count: (state.log.total_lines)(),
                    onexport: move |_| bridge.export((state.ui.show_timestamps)()),
                    onclear: move |_| {
                        bridge.clear();
                        state.clear_logs();
                        vs.start_index.set(0);
                        state.success("Logs Cleared");
                    },
                    ontoggle_autoscroll: move |_| state.ui.toggle_autoscroll(),
                }

                LogViewport {
                    total_height: vs.total_height,
                    offset_top: vs.offset_top,
                    onmounted_console: move |evt: MountedEvent| {
                        let handle = evt.data();
                        let h_clone = handle.clone();
                        spawn(async move {
                            if let Ok(rect) = h_clone.get_client_rect().await {
                                vs.console_height.set(rect.height());
                            }
                        });
                        vs.console_handle.set(Some(handle));
                    },
                    onscroll: move |_: ScrollEvent| {
                        let handle = vs.console_handle.peek().as_ref().cloned();
                        let total_lines = (state.log.total_lines)();
                        spawn(async move {
                            if let Some(handle) = handle {
                                if let Ok(offset) = handle.get_scroll_offset().await {
                                    let (new_index, is_at_bottom) = calculate_scroll_state(
                                        offset.y,
                                        *vs.console_height.read(),
                                        total_lines,
                                        vs.scale_factor,
                                    );
                                    if (vs.start_index)() != new_index {
                                        vs.start_index.set(new_index);
                                    }
                                    if (state.ui.autoscroll)() != is_at_bottom {
                                        state.ui.set_autoscroll(is_at_bottom);
                                    }
                                }
                            }
                        });
                    },
                    onmounted_sentinel: move |evt: MountedEvent| vs.sentinel_handle.set(Some(evt.data())),
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
