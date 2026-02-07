use crate::components::monitor::hooks::effects::{use_search_sync, use_settings_sync};
use crate::components::monitor::monitor_header::MonitorHeader;
use crate::components::monitor::monitor_viewport::MonitorViewport;
use crate::components::ui::buttons::ResumeScrollButton;
use crate::components::ui::console::ConsoleFrame;
use crate::hooks::use_worker_controller;
use crate::state::AppState;
use dioxus::prelude::*;

use crate::components::monitor::hooks::virtual_scroll::use_virtual_scroll;

#[component]
pub fn Monitor() -> Element {
    let state = use_context::<AppState>();
    let bridge = use_worker_controller();
    let mut vs = use_virtual_scroll();

    // Initial log sync and effects
    use_settings_sync(bridge);
    use_search_sync(bridge);

    rsx! {
        ConsoleFrame {
            MonitorHeader {
                autoscroll: (state.ui.autoscroll)(),
                count: (state.log.total_lines)(),
                onexport: move |_| bridge.export((state.ui.show_timestamps)()),
                onclear: move |_| {
                    bridge.clear();
                    state.log.clear();
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
                ResumeScrollButton {
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
