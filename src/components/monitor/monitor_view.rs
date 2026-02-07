use crate::components::monitor::hooks::effects::{use_search_sync, use_settings_sync};
use crate::components::monitor::monitor_viewport::MonitorViewport;
use crate::hooks::use_worker_controller;
use crate::state::AppState;
use dioxus::prelude::*;

use crate::components::monitor::hooks::virtual_scroll::use_virtual_scroll;
use crate::components::monitor::monitor_header::MonitorHeader;

#[component]
pub fn Monitor() -> Element {
    let state = use_context::<AppState>();
    let bridge = use_worker_controller();
    let mut vs = use_virtual_scroll();

    // Initial log sync and effects
    use_settings_sync(bridge);
    use_search_sync(bridge);

    rsx! {
        main { class: "flex-1 min-h-0 mx-4 mb-0 mt-0 relative group/monitor",
            div { class: "absolute inset-0 bg-console-bg rounded-t-2xl border-t border-x border-[#222629] shadow-[inset_0_0_20px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col",
                div { class: "absolute inset-0 scanlines opacity-20 pointer-events-none z-10" }

                MonitorHeader {
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

                MonitorViewport {
                    total_height: vs.total_height,
                    offset_top: vs.offset_top,
                    onmounted_console: move |evt: MountedEvent| {
                        vs.console_handle.set(Some(evt.data()));
                    },
                    onscroll: move |_: ScrollEvent| {
                        vs.scroll_task.restart();
                    },
                    onmounted_sentinel: move |evt: MountedEvent| vs.sentinel_handle.set(Some(evt.data())),
                }

                if !(state.ui.autoscroll)() {
                    crate::components::ui::buttons::ResumeScrollButton {
                        onclick: move |_| {
                            web_sys::window()
                                .and_then(|win| win.document())
                                .and_then(|doc| doc.get_element_by_id("console-output"))
                                .map(|el| el.set_scroll_top(el.scroll_height()));
                        },
                    }
                }
            }
        }
    }
}
