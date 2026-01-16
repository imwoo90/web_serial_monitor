use crate::components::common::{CustomSelect, IconButton};
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    let mut state = use_context::<AppState>();
    let is_open = (state.show_settings)();

    let settings_icon_class = if is_open {
        "text-[20px] transition-all duration-300 rotate-45"
    } else {
        "text-[20px] transition-all duration-300"
    };

    rsx! {
        header { class: "shrink-0 pt-6 px-5 pb-2 flex flex-col gap-4 z-20",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-3",
                    div { class: "h-8 w-8 rounded-lg bg-linear-to-br from-primary to-blue-600 flex items-center justify-center shadow-lg shadow-primary/20",
                        span { class: "material-symbols-outlined text-black text-[20px] font-bold", "terminal" }
                    }
                    h1 { class: "text-xl font-bold tracking-tight leading-none",
                        "Serial"
                        br {}
                        span { class: "text-gray-500 text-base font-medium", "Monitor v1.0" }
                    }
                }
                button { class: "group relative flex items-center gap-2 bg-[#1A1D1F] border border-white/5 hover:border-primary/50 pl-3 pr-4 py-2 rounded-full transition-all duration-300 active:scale-95 shadow-lg",
                    div { class: "relative flex h-2.5 w-2.5",
                        span { class: "animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-0 group-hover:opacity-100 transition-opacity duration-300" }
                        span { class: "relative inline-flex rounded-full h-2.5 w-2.5 bg-gray-500 group-hover:bg-primary transition-colors duration-300" }
                    }
                    span { class: "text-sm font-bold text-gray-300 group-hover:text-white transition-colors", "Connect" }
                }
            }
            div { class: "flex gap-2",
                div { class: "flex items-center gap-2 px-3 py-2 bg-surface rounded-lg border border-white/5 shadow-sm",
                    span { class: "material-symbols-outlined text-[#29A329] text-[18px]", "usb" }
                    span { class: "text-xs font-bold text-gray-400", "COM3" }
                }

                CustomSelect {
                    options: vec!["9600", "19200", "38400", "57600", "115200"],
                    selected: state.baud_rate,
                    onchange: move |val| state.baud_rate.set(val),
                    class: "flex-1"
                }

                IconButton {
                    icon: "settings",
                    active: is_open,
                    class: "w-10 h-10 bg-surface border border-white/5 rounded-lg",
                    icon_class: settings_icon_class,
                    onclick: move |_| {
                        let current = (state.show_settings)();
                        state.show_settings.set(!current);
                    },
                    title: "Settings"
                }
            }
        }
    }
}
