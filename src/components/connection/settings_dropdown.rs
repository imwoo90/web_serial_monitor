use crate::components::ui::CustomSelect;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn SettingsDropdown(is_open: bool, onclose: EventHandler<()>) -> Element {
    let state = use_context::<AppState>();

    // Signals removed, using direct state access in rsx!

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
                        selected: (state.serial.data_bits)().to_string(),
                        onchange: move |val: String| {
                            if let Ok(b) = val.parse::<u8>() {
                                state.serial.set_data_bits(b);
                            }
                        },
                        disabled: state.conn.is_connected(),
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Stop Bits"
                    }
                    CustomSelect {
                        options: vec!["1", "2"],
                        selected: if (state.serial.stop_bits)() == 1 { "1".to_string() } else { "2".to_string() },
                        onchange: move |val: String| {
                            if let Ok(b) = val.parse::<u8>() {
                                state.serial.set_stop_bits(b);
                            }
                        },
                        disabled: state.conn.is_connected(),
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Parity"
                    }
                    CustomSelect {
                        options: vec!["None", "Even", "Odd"],
                        selected: match (state.serial.parity)() {
                            crate::state::Parity::None => "None".to_string(),
                            crate::state::Parity::Even => "Even".to_string(),
                            crate::state::Parity::Odd => "Odd".to_string(),
                        },
                        onchange: move |val: String| {
                            let p = match val.as_str() {
                                "Even" => crate::state::Parity::Even,
                                "Odd" => crate::state::Parity::Odd,
                                _ => crate::state::Parity::None,
                            };
                            state.serial.set_parity(p);
                        },
                        disabled: state.conn.is_connected(),
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Flow Control"
                    }
                    CustomSelect {
                        options: vec!["None", "Hardware"],
                        selected: match (state.serial.flow_control)() {
                            crate::state::FlowControl::None => "None".to_string(),
                            crate::state::FlowControl::Hardware => "Hardware".to_string(),
                        },
                        onchange: move |val: String| {
                            let f = match val.as_str() {
                                "Hardware" => crate::state::FlowControl::Hardware,
                                _ => crate::state::FlowControl::None,
                            };
                            state.serial.set_flow_control(f);
                        },
                        disabled: state.conn.is_connected(),
                    }
                }
            }
        }
    }
}
