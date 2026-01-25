use crate::components::common::CustomSelect;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn SettingsDropdown(is_open: bool, onclose: EventHandler<()>) -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        if is_open {
            div {
                class: "fixed inset-0 z-40 cursor-default",
                onclick: move |_| onclose.call(()),
            }
        }
        div {
            class: "absolute top-full right-6 mt-2 w-80 bg-[#16181a] rounded-xl border border-[#2a2e33] shadow-2xl transition-all duration-300 z-50 origin-top-right text-left",
            class: if is_open { "opacity-100 visible scale-100 translate-y-0 p-4" } else { "opacity-0 invisible scale-95 -translate-y-2 p-0 overflow-hidden h-0" },
            div { class: "grid grid-cols-2 gap-x-3 gap-y-4",
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Data Bits"
                    }
                    CustomSelect {
                        options: vec!["5", "6", "7", "8"],
                        selected: state.data_bits,
                        onchange: move |val| state.data_bits.set(val),
                        disabled: (state.is_connected)(),
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Stop Bits"
                    }
                    CustomSelect {
                        options: vec!["1", "1.5", "2"],
                        selected: state.stop_bits,
                        onchange: move |val| state.stop_bits.set(val),
                        disabled: (state.is_connected)(),
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Parity"
                    }
                    CustomSelect {
                        options: vec!["None", "Even", "Odd", "Mark", "Space"],
                        selected: state.parity,
                        onchange: move |val| state.parity.set(val),
                        disabled: (state.is_connected)(),
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Flow Control"
                    }
                    CustomSelect {
                        options: vec!["None", "Hardware", "Software"],
                        selected: state.flow_control,
                        onchange: move |val| state.flow_control.set(val),
                        disabled: (state.is_connected)(),
                    }
                }
            }
        }
    }
}
