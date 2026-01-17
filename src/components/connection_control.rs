use crate::components::common::{CustomSelect, IconButton};
use crate::components::console::types::WorkerMsg;
use crate::serial;
use crate::state::{AppState, SerialPortWrapper};
use crate::utils::{format_hex, LineParser};
use chrono::Local;
use dioxus::prelude::*;

#[component]
pub fn ConnectionControl() -> Element {
    let mut state = use_context::<AppState>();
    let is_open = (state.show_settings)();

    let settings_icon_class = if is_open {
        "text-[20px] transition-all duration-300 rotate-45"
    } else {
        "text-[20px] transition-all duration-300"
    };

    rsx! {
        div { class: "flex items-center gap-3 h-full",
            // Port Info
            div { class: "flex items-center gap-2 px-3 py-1.5 bg-[#16181a] rounded-lg border border-[#2a2e33] h-9",
                if (state.is_connected)() {
                    span { class: "material-symbols-outlined text-emerald-500 text-[18px]",
                        "usb"
                    }
                    span { class: "text-xs font-bold text-emerald-500 font-mono", "Connected" }
                } else {
                    span { class: "material-symbols-outlined text-gray-500 text-[18px]",
                        "usb_off"
                    }
                    span { class: "text-xs font-bold text-gray-500 font-mono", "No Device" }
                }
            }

            // Baud Rate
            div { class: "w-32",
                CustomSelect {
                    options: vec!["9600", "19200", "38400", "57600", "115200"],
                    selected: state.baud_rate,
                    onchange: move |val| state.baud_rate.set(val),
                    class: "w-full",
                    disabled: (state.is_connected)(),
                }
            }

            // Settings Button
            IconButton {
                icon: "settings",
                active: is_open,
                class: "w-9 h-9 bg-[#16181a] border border-[#2a2e33] rounded-lg hover:border-primary/50 hover:text-white transition-colors",
                icon_class: settings_icon_class,
                onclick: move |_| {
                    let current = (state.show_settings)();
                    state.show_settings.set(!current);
                },
                title: "Settings",
            }

            // Connect Button
            button {
                class: if (state.is_connected)() { "group relative flex items-center gap-2 bg-red-500/80 hover:bg-red-500 border border-red-500/50 pl-3 pr-4 py-1.5 rounded-lg transition-all duration-300 active:scale-95 shadow-lg shadow-red-500/20 ml-2" } else { "group relative flex items-center gap-2 bg-primary hover:bg-primary-hover border border-primary/50 pl-3 pr-4 py-1.5 rounded-lg transition-all duration-300 active:scale-95 shadow-lg shadow-primary/20 ml-2" },
                onclick: move |_| {
                    if (state.is_connected)() {
                        spawn(async move {
                            if let Some(wrapper) = (state.port)() {
                                let _ = serial::close_port(&wrapper.0).await;
                                state.port.set(None);
                                state.is_connected.set(false);
                                state.info("Disconnected");
                            }
                        });
                    } else {
                        spawn(async move {
                            if let Ok(port) = serial::request_port().await {
                                let baud = (state.baud_rate)().parse().unwrap_or(115200);
                                let data_bits = (state.data_bits)().parse().unwrap_or(8);
                                let stop_bits = if (state.stop_bits)() == "2" { 2 } else { 1 };

                                if serial::open_port(

                                        &port,
                                        baud,
                                        data_bits,
                                        stop_bits,
                                        (state.parity)(),
                                        (state.flow_control)(),
                                    )
                                    .await
                                    .is_ok()
                                {
                                    state.port.set(Some(SerialPortWrapper(port.clone())));
                                    state.is_connected.set(true);
                                    state.success("Connected");
                                    let mut parser = LineParser::new();
                                    serial::read_loop(
                                            port,
                                            move |data| {
                                                if (state.is_hex_view)() {
                                                    let hex_string = format_hex(&data);
                                                    if let Some(w) = state.log_worker.peek().as_ref() {
                                                        let timestamp = Local::now()
                                                            .format("[%H:%M:%S%.3f] ")
                                                            .to_string();
                                                        let log_entry = format!("{}{}", timestamp, hex_string);
                                                        let msg = WorkerMsg::AppendLog(log_entry);
                                                        let _ = w
                                                            .post_message(&serde_wasm_bindgen::to_value(&msg).unwrap());
                                                    }
                                                } else {
                                                    let mode = (state.rx_line_ending)();
                                                    parser.set_mode(mode);
                                                    let chunk = String::from_utf8_lossy(&data);
                                                    let lines = parser.push(&chunk);
                                                    if let Some(w) = state.log_worker.peek().as_ref() {
                                                        for line in lines {
                                                            let timestamp = Local::now()
                                                                .format("[%H:%M:%S%.3f] ")
                                                                .to_string();
                                                            let log_entry = format!("{}{}", timestamp, line);
                                                            let msg = WorkerMsg::AppendLog(log_entry);
                                                            let _ = w
                                                                .post_message(&serde_wasm_bindgen::to_value(&msg).unwrap());
                                                        }
                                                    }
                                                }
                                            },
                                            move |_| {
                                                state.is_connected.set(false);
                                                state.port.set(None);
                                                state.error("Connection Lost");
                                            },
                                        )
                                        .await;
                                } else {
                                    state.error("Failed to Open Port");
                                }
                            }
                        });
                    }
                },
                div { class: "relative flex h-2 w-2",
                    span {
                        class: "animate-ping absolute inline-flex h-full w-full rounded-full opacity-75",
                        class: if (state.is_connected)() { "bg-white" } else { "bg-white" },
                    }
                    span { class: "relative inline-flex rounded-full h-2 w-2 bg-white" }
                }
                span {
                    class: "text-xs font-bold transition-colors uppercase tracking-wide",
                    class: if (state.is_connected)() { "text-white" } else { "text-black group-hover:text-black/80" },
                    if (state.is_connected)() {
                        "Disconnect"
                    } else {
                        "Connect"
                    }
                }
            }

            // Settings Dropdown Panel
            if is_open {
                div {
                    class: "fixed inset-0 z-40 cursor-default",
                    onclick: move |_| state.show_settings.set(false),
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
}
