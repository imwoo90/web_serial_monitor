use crate::components::connection::{BaudRatePicker, PortStatus, SettingsDropdown};
use crate::components::ui::IconButton;
use crate::hooks::use_serial_controller;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn ConnectionControl() -> Element {
    let state = use_context::<AppState>();
    let controller = use_serial_controller();
    let is_open = (state.ui.show_settings)();

    let settings_icon_class = if is_open {
        "text-[20px] transition-all duration-300 rotate-45"
    } else {
        "text-[20px] transition-all duration-300"
    };

    rsx! {
        div { class: "flex items-center gap-3 h-full",
            // Port Info
            PortStatus { connected: state.conn.is_connected() }

            // Baud Rate
            BaudRatePicker {
                baud_rate: state.serial.baud_rate,
                disabled: state.conn.is_connected(),
                onchange: move |val| state.serial.set_baud_rate(val),
            }

            // Settings Button
            IconButton {
                icon: "settings",
                active: is_open,
                class: "w-9 h-9 bg-[#16181a] border border-[#2a2e33] rounded-lg hover:border-primary/50 hover:text-white transition-colors",
                icon_class: settings_icon_class,
                onclick: move |_| state.ui.toggle_settings(),
                title: "Settings",
            }

            // Test Mode Button
            if cfg!(debug_assertions) {
                button {
                    class: if (state.conn.is_simulating)() { "flex items-center justify-center w-9 h-9 bg-yellow-500/80 hover:bg-yellow-500 border border-yellow-500/50 rounded-lg transition-all active:scale-95 shadow-lg shadow-yellow-500/20 text-white gap-2" } else { "flex items-center justify-center w-9 h-9 bg-[#16181a] border border-[#2a2e33] rounded-lg hover:border-yellow-500/50 hover:text-yellow-500 transition-colors text-gray-400 gap-2" },
                    onclick: move |_| {
                        if (state.conn.is_simulating)() {
                            controller.stop_simulation();
                        } else {
                            controller.start_simulation();
                        }
                    },
                    title: "Test Mode",
                    span { class: "material-symbols-outlined text-[18px]", "bug_report" }
                }
            }

            // Connect Button
            button {
                class: if state.conn.is_connected() { "group relative flex items-center gap-2 bg-red-500/80 hover:bg-red-500 border border-red-500/50 pl-3 pr-4 py-1.5 rounded-lg transition-all duration-300 active:scale-95 shadow-lg shadow-red-500/20 ml-2" } else { "group relative flex items-center gap-2 bg-primary hover:brightness-110 border border-primary/50 pl-3 pr-4 py-1.5 rounded-lg transition-all duration-300 active:scale-95 shadow-lg shadow-primary/20 ml-2" },
                onclick: move |_| {
                    if state.conn.is_connected() {
                        controller.disconnect();
                    } else {
                        controller.connect();
                    }
                },
                div { class: "relative flex h-2 w-2",
                    span { class: "animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 bg-white" }
                    span { class: "relative inline-flex rounded-full h-2 w-2 bg-white" }
                }
                span {
                    class: "text-xs font-bold transition-colors uppercase tracking-wide",
                    class: if state.conn.is_connected() { "text-white" } else { "text-black group-hover:text-black/80" },
                    if state.conn.is_connected() {
                        "Disconnect"
                    } else {
                        "Connect"
                    }
                }
            }

            // Settings Dropdown Panel
            SettingsDropdown {
                is_open,
                onclose: move |_| {
                    if (state.ui.show_settings)() {
                        state.ui.toggle_settings();
                    }
                },
            }
        }
    }
}
