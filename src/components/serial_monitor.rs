use crate::components::ui::ToastContainer;
use dioxus::prelude::*;

use super::monitor::{FilterBar, InputBar, MacroBar, Monitor};
use super::terminal::{AutoDisposeTerminal, Xterm};
use crate::components::header::Header;
use crate::hooks::use_worker_controller;
use crate::state::ViewMode;

#[component]
pub fn SerialMonitor() -> Element {
    let app_state = crate::state::use_provide_app_state();
    let bridge = use_worker_controller();
    let view_mode = app_state.ui.view_mode;
    let term_instance = use_signal(|| None::<AutoDisposeTerminal>);

    use_effect(move || {
        let mode = view_mode();
        bridge.set_mode(mode);
        match mode {
            ViewMode::Monitoring => {
                bridge.clear();
                app_state.log.clear();
            }
            ViewMode::Terminal => {
                app_state.terminal.clear();
            }
        }
    });

    let toasts = app_state.log.toasts;

    rsx! {
        div { class: "bg-background-dark h-screen w-full font-display text-white selection:bg-primary/30 selection:text-primary overflow-x-auto overflow-y-hidden",
            div { class: "flex flex-col h-full min-w-[600px]",
                Header {}
                if view_mode() == ViewMode::Monitoring {
                    InputBar {}
                    FilterBar {}
                    Monitor {}
                } else {
                    div { class: "relative flex-1 flex flex-col min-h-0 w-full",
                        Xterm { term_instance }
                        if !*app_state.terminal.autoscroll.read() {
                            crate::components::monitor::monitor_view::ResumeScrollButton {
                                onclick: move |_| {
                                    if let Some(term) = term_instance.read().as_ref() {
                                        term.scroll_to_bottom();
                                    }
                                },
                            }
                        }
                    }
                }
                MacroBar {}
                ToastContainer { toasts }
            }
        }
    }
}
