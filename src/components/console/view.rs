use crate::components::console::hooks::effects::{use_search_sync, use_settings_sync};
use crate::components::console::viewport::LogViewport;
use crate::hooks::use_worker_controller;
use crate::state::AppState;
use dioxus::prelude::*;

use crate::components::console::console_header::ConsoleHeader;
use crate::components::console::hooks::virtual_scroll::use_virtual_scroll;

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
