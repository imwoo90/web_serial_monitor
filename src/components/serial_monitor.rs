use crate::components::ui::ToastContainer;
use dioxus::prelude::*;

use super::console::{Console, FilterBar, InputBar, MacroBar};
use crate::components::header::Header;
use crate::hooks::use_worker_controller;
use crate::state::ViewMode;

#[component]
pub fn SerialMonitor() -> Element {
    let app_state = crate::state::use_provide_app_state();

    // Lifecycle/Worker Hook
    use_worker_controller();

    let toasts = app_state.log.toasts;

    let view_mode = app_state.ui.view_mode;

    rsx! {
        div { class: "bg-background-dark h-screen w-full font-display text-white selection:bg-primary/30 selection:text-primary overflow-x-auto overflow-y-hidden",
            div { class: "flex flex-col h-full min-w-[600px]",
                Header {}
                if view_mode() == ViewMode::Monitoring {
                    InputBar {}
                    FilterBar {}
                    Console {}
                } else {
                    // Placeholder for Terminal View
                    div { class: "flex-1 w-full bg-black text-white p-4 font-mono flex items-center justify-center",
                        "Terminal View (Coming Soon)"
                    }
                }
                MacroBar {}
                ToastContainer { toasts }
            }
        }
    }
}
