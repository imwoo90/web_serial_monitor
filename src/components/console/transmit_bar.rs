use crate::state::{AppState, LineEnding};
use crate::utils::serial;
use crate::utils::{format_hex_input, parse_hex_string, CommandHistory};
use dioxus::prelude::*;
use futures_util::StreamExt;
use js_sys::Uint8Array;

#[component]
pub fn TransmitBar() -> Element {
    let mut state = use_context::<AppState>();
    let mut input_value = use_signal(String::new);
    let mut history = use_signal(CommandHistory::load);
    let mut history_index = use_signal(|| None::<usize>);
    let mut is_hex_input = use_signal(|| false);
    let bridge = crate::hooks::use_worker_controller();

    let send_task = use_coroutine(move |mut rx| async move {
        while rx.next().await.is_some() {
            let text = input_value();
            let ending = *state.serial.tx_line_ending.peek();

            if text.is_empty() && matches!(ending, LineEnding::None) {
                continue;
            }

            let is_hex = is_hex_input();
            let local_echo = *state.serial.tx_local_echo.peek();
            let port = state.conn.port.peek().as_ref().cloned();

            let mut data = if is_hex {
                if text.is_empty() {
                    Vec::new()
                } else {
                    match parse_hex_string(&text) {
                        Ok(d) => d,
                        Err(e) => {
                            if let Some(w) = web_sys::window() {
                                let _ = w.alert_with_message(&format!("Hex Error: {}", e));
                            }
                            continue;
                        }
                    }
                }
            } else {
                text.clone().into_bytes()
            };

            match ending {
                LineEnding::NL => data.push(b'\n'),
                LineEnding::CR => data.push(b'\r'),
                LineEnding::NLCR => {
                    data.push(b'\r');
                    data.push(b'\n');
                }
                _ => {}
            }

            if let Some(conn_port) = port {
                if serial::send_data(&conn_port, &data).await.is_ok() {
                    if local_echo {
                        let array = Uint8Array::from(data.as_slice());
                        bridge.append_chunk(array, false);
                    }
                    input_value.set(String::new());
                    if !text.is_empty() {
                        history.write().add(text);
                    }
                    history_index.set(None);
                }
            }
        }
    });

    rsx! {
        div { class: "flex-1 relative flex gap-2 min-w-0",
            div { class: "relative flex-1 group",
                input {
                    class: "w-full h-full bg-[#0d0f10] text-sm text-white placeholder-gray-600 px-4 pr-16 rounded-lg border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none shadow-inset-input transition-all font-mono",
                    placeholder: "Send command...",
                    "type": "text",
                    value: "{input_value}",
                    id: "transmit-input",
                    oninput: move |evt| {
                        if is_hex_input() {
                            input_value.set(format_hex_input(&evt.value()));
                        } else {
                            input_value.set(evt.value());
                        }
                    },
                    onkeydown: move |evt| {
                        match evt.key() {
                            Key::Enter => send_task.send(()),
                            Key::ArrowUp => {
                                let h = history.read();
                                if h.len() > 0 {
                                    let idx = history_index()
                                        .map(|i| if i > 0 { i - 1 } else { 0 })
                                        .unwrap_or(h.len() - 1);
                                    history_index.set(Some(idx));
                                    if let Some(c) = h.get_at(idx) {
                                        input_value.set(c.clone());
                                    }
                                }
                                evt.prevent_default();
                            }
                            Key::ArrowDown => {
                                if let Some(i) = history_index() {
                                    let h = history.read();
                                    if i + 1 >= h.len() {
                                        history_index.set(None);
                                        input_value.set(String::new());
                                    } else {
                                        history_index.set(Some(i + 1));
                                        if let Some(c) = h.get_at(i + 1) {
                                            input_value.set(c.clone());
                                        }
                                    }
                                }
                                evt.prevent_default();
                            }
                            _ => {}
                        }
                    },
                }
                div { class: "absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1",
                    button {
                        class: "px-1.5 py-0.5 rounded text-[10px] font-bold border transition-colors",
                        class: if (state.serial.tx_local_echo)() { "bg-emerald-500/20 text-emerald-500 border-emerald-500/30" } else { "text-gray-500 border-transparent hover:text-gray-300" },
                        onclick: move |_| state.serial.tx_local_echo.set(!(state.serial.tx_local_echo)()),
                        title: "Local Echo: Show sent commands in log",
                        "ECHO"
                    }
                    button {
                        class: "px-1.5 py-0.5 rounded text-[10px] font-bold border transition-colors",
                        class: if is_hex_input() { "bg-primary/20 text-primary border-primary/30" } else { "text-gray-500 border-transparent hover:text-gray-300" },
                        onclick: move |_| is_hex_input.set(!is_hex_input()),
                        title: "HEX Input Mode",
                        "HEX"
                    }
                }
            }

            button {
                class: "h-full aspect-square bg-primary text-surface rounded-lg flex items-center justify-center hover:bg-white transition-all hover:shadow-[0_0_15px_rgba(255,255,255,0.4)] active:scale-95 group",
                onclick: move |_| send_task.send(()),
                span { class: "material-symbols-outlined text-[20px] group-hover:rotate-45 transition-transform duration-300",
                    "send"
                }
            }
        }
    }
}
