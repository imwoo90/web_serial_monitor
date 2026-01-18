use crate::components::common::{CustomInputSelect, CustomSelect, IconButton};
use crate::components::console::types::WorkerMsg;
use crate::serial;
use crate::state::{AppState, SerialPortWrapper};
use crate::utils::send_chunk_to_worker;
use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::ReadableStreamDefaultReader;

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
                CustomInputSelect {
                    options: vec![
                        "1200",
                        "2400",
                        "4800",
                        "9600",
                        "19200",
                        "38400",
                        "57600",
                        "115200",
                        "230400",
                        "460800",
                        "921600",
                    ],
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

            // Test Mode Button
            button {
                class: if (state.is_simulating)() { "flex items-center justify-center w-9 h-9 bg-yellow-500/80 hover:bg-yellow-500 border border-yellow-500/50 rounded-lg transition-all active:scale-95 shadow-lg shadow-yellow-500/20 text-white gap-2" } else { "flex items-center justify-center w-9 h-9 bg-[#16181a] border border-[#2a2e33] rounded-lg hover:border-yellow-500/50 hover:text-yellow-500 transition-colors text-gray-400 gap-2" },
                onclick: move |_| {
                    let current = (state.is_simulating)();
                    let next = !current;
                    state.is_simulating.set(next);

                    if next {
                        state.info("Simulation Started");

                        // Clear logs if starting
                        if let Some(w) = state.log_worker.peek().as_ref() {
                            let _ = w
                                .post_message(
                                    &serde_wasm_bindgen::to_value(&WorkerMsg::Clear).unwrap(),
                                );
                        }
                        let worker_sig = state.log_worker;
                        let sim_sig = state.is_simulating;
                        let hex_sig = state.is_hex_view;

                        spawn(async move {
                            loop {
                                if !sim_sig() {
                                    break;
                                }
                                if let Some(w) = worker_sig.peek().as_ref() {
                                    // Generate dummy content
                                    let rnd = js_sys::Math::random();
                                    let content = if rnd < 0.1 {
                                        format!(
                                            "Error: System overheat at {:.1}°C\n",
                                            80.0 + rnd * 20.0,
                                        )
                                    } else if rnd < 0.3 {
                                        format!(
                                            "Warning: Voltage fluctuation detected: {:.2}V\n",
                                            3.0 + rnd,
                                        )
                                    } else {
                                        format!(
                                            "Info: Sensor reading: A={:.2}, B={:.2}, C={:.2}\n",
                                            rnd * 100.0,
                                            rnd * 50.0,
                                            rnd * 10.0,
                                        )
                                    };

                                    // Worker now handles formatting. Just send raw bytes.
                                    let is_hex = hex_sig();
                                    send_chunk_to_worker(w, content.as_bytes(), is_hex);
                                }
                                gloo_timers::future::TimeoutFuture::new(1).await;
                            }
                        });
                    } else {
                        state.info("Simulation Stopped");
                    }
                },
                title: "Test Mode",
                span { class: "material-symbols-outlined text-[18px]", "bug_report" }
            }

            // Connect Button
            button {
                class: if (state.is_connected)() { "group relative flex items-center gap-2 bg-red-500/80 hover:bg-red-500 border border-red-500/50 pl-3 pr-4 py-1.5 rounded-lg transition-all duration-300 active:scale-95 shadow-lg shadow-red-500/20 ml-2" } else { "group relative flex items-center gap-2 bg-primary hover:bg-primary-hover border border-primary/50 pl-3 pr-4 py-1.5 rounded-lg transition-all duration-300 active:scale-95 shadow-lg shadow-primary/20 ml-2" },
                onclick: move |_| {
                    if (state.is_connected)() {
                        spawn(async move {
                            // Cancel reader first
                            if let Some(reader_wrapper) = (state.reader)() {
                                let _ = serial::cancel_reader(&reader_wrapper.0).await;
                                state.reader.set(None);
                            }

                            // Then close port
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
                                    // Clear logs before connecting
                                    if let Some(w) = state.log_worker.peek().as_ref() {
                                        let _ = w
                                            .post_message(
                                                &serde_wasm_bindgen::to_value(&WorkerMsg::Clear).unwrap(),
                                            );
                                    }
                                    state.port.set(Some(SerialPortWrapper(port.clone())));
                                    state.is_connected.set(true);

                                    // Create reader and store it
                                    let readable = port.readable();
                                    let reader = readable.get_reader();
                                    let reader: ReadableStreamDefaultReader = reader.unchecked_into();
                                    state.reader.set(Some(crate::state::ReaderWrapper(reader.clone())));

                                    state.success("Connected");

                                    serial::read_loop(
                                            reader,
                                            move |data| {
                                                // data is Vec<u8>
                                                if let Some(w) = state.log_worker.peek().as_ref() {
                                                    let is_hex = (state.is_hex_view)();
                                                    send_chunk_to_worker(w, &data, is_hex);
                                                }
                                            },
                                            move |_| {
                                                // If we manually disconnected, is_connected will be false by the time this runs (maybe?)
                                                // But if we just cancelled the reader, this callback might run on error or not?
                                                // The on_error callback is called if read() errors.
                                                // If read() is cancelled, it resolves with done=true.
                                                // My read_loop logic calls break on done, NOT on_error.
                                                // So this callback is only called on actual errors.
                                                state.is_connected.set(false);
                                                state.port.set(None);
                                                state.reader.set(None);
                                                state.error("Connection Lost");
                                            },
                                        )
                                        .await;

                                    // If we are here, loop finished.
                                    if (state.is_connected)() {
                                         // It means it wasn't a manual disconnect (which sets is_connected=false before cancelling reader).
                                         // Wait, manual disconnect:
                                         // 1. User clicks.
                                         // 2. Spawn async task.
                                         // 3. cancel_reader().
                                         // 4. state.reader.set(None).
                                         // 5. ... close port ...
                                         // 6. state.is_connected.set(false).

                                         // Meanwhile, read_loop breaks.
                                         // It finishes.
                                         // The "await" in this spawn block finishes.

                                         // We need to avoid "Connection Closed" message if manual disconnect is in progress.
                                         // But `is_connected` is still true until `close_port` finishes?
                                         // No, `cancel_reader` finishes, then we set `is_connected=false`.

                                         // Race condition:
                                         // `read_loop` finishes when `cancel` resolves.
                                         // The `spawn` block for `Connect` resumes after `await`.
                                         // At this point, we are in the `Connect`'s async block.
                                         // The `Disconnect` async block is running in parallel.

                                         // If `Disconnect` hasn't reached `is_connected.set(false)` yet, we might show "Connection Closed".

                                         // Solution: Check `state.reader`. If it's None, it implies we are disconnecting manually?
                                         // We set `reader` to None in Disconnect handler.
                                         // So:
                                         if (state.reader)().is_some() {
                                             state.is_connected.set(false);
                                             state.port.set(None);
                                             state.reader.set(None);
                                             state.info("Connection Closed");
                                         }
                                    }
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
