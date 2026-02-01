use crate::state::{AppState, LineEnding};
use crate::utils::serial;
use crate::utils::{format_hex_input, parse_hex_string, MacroStorage};
use dioxus::prelude::*;

#[component]
pub fn MacroBar() -> Element {
    let state = use_context::<AppState>();
    let mut storage = use_signal(MacroStorage::load);
    let mut show_form = use_signal(|| false);

    let mut new_label = use_signal(String::new);
    let mut new_cmd = use_signal(String::new);
    let mut new_hex = use_signal(|| false);

    let mut current_macro = use_signal(|| None::<(String, bool)>);

    let mut macro_task = use_resource(move || {
        let macro_data = current_macro();
        let port = (state.conn.port).peek().as_ref().cloned();
        let ending = (state.serial.tx_line_ending)();

        async move {
            if let Some((cmd, is_hex)) = macro_data {
                let mut data = if is_hex {
                    match parse_hex_string(&cmd) {
                        Ok(d) => d,
                        Err(e) => {
                            if let Some(w) = web_sys::window() {
                                let _ = w.alert_with_message(&format!("Macro Hex Error: {}", e));
                            }
                            return;
                        }
                    }
                } else {
                    cmd.into_bytes()
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
                    let _ = serial::send_data(&conn_port, &data).await;
                }
            }
        }
    });

    rsx! {
        div { class: "flex gap-2 p-2 bg-background-dark border-t border-[#2a2e33] min-h-[40px] items-center overflow-x-auto",
            div { class: "flex gap-2 flex-1 items-center",
                for item in storage.read().get_items() {
                    button {
                        key: "{item.id}",
                        class: "shrink-0 px-3 py-1 bg-[#2a2e33] hover:bg-primary hover:text-white rounded text-xs font-mono transition-colors border border-gray-700 select-none whitespace-nowrap",
                        onclick: move |_| {
                            current_macro.set(Some((item.command.clone(), item.is_hex)));
                            macro_task.restart();
                        },
                        oncontextmenu: move |evt| {
                            evt.prevent_default();
                            storage.write().remove(item.id);
                        },
                        title: "Right-click to remove",
                        "{item.label}"
                    }
                }

                // Add Button
                button {
                    class: "shrink-0 w-6 h-6 flex items-center justify-center bg-[#1a1c1e] text-gray-400 hover:text-white rounded text-xs border border-dashed border-gray-700 hover:border-gray-500 transition-colors",
                    onclick: move |_| show_form.set(!show_form()),
                    title: "Add Macro",
                    "+"
                }
            }

            // GitHub Link (Moved from Footer)
            div { class: "shrink-0 flex items-center gap-4 ml-auto px-2",
                a {
                    class: "text-gray-500 hover:text-primary transition-colors flex items-center gap-1.5 group text-[11px]",
                    href: "https://github.com/imwoo90/web_serial_monitor",
                    target: "_blank",
                    span { class: "material-symbols-outlined text-[14px]", "code" }
                    span { "GitHub" }
                }
            }

            // Form Modal
            if show_form() {
                div { class: "fixed inset-0 z-40 flex items-center justify-center bg-black/50 backdrop-blur-sm",
                    div { class: "bg-[#16181a] p-4 rounded-xl border border-[#2a2e33] w-80 shadow-2xl",
                        h3 { class: "text-sm font-bold text-gray-300 mb-3", "Add Quick Command" }
                        div { class: "space-y-3",
                            div {
                                label { class: "block text-[10px] uppercase text-gray-500 font-bold mb-1",
                                    "Label"
                                }
                                input {
                                    class: "w-full bg-[#0d0f10] text-white p-2 rounded border border-[#2a2e33] text-xs focus:border-primary/50 outline-none",
                                    placeholder: "e.g. Reboot",
                                    value: "{new_label}",
                                    oninput: move |e| new_label.set(e.value()),
                                    autofocus: true,
                                }
                            }
                            div {
                                label { class: "block text-[10px] uppercase text-gray-500 font-bold mb-1",
                                    "Command"
                                }
                                input {
                                    class: "w-full bg-[#0d0f10] text-white p-2 rounded border border-[#2a2e33] text-xs font-mono focus:border-primary/50 outline-none",
                                    placeholder: "e.g. AT+RST",
                                    value: "{new_cmd}",
                                    oninput: move |e| {
                                        if new_hex() {
                                            let formatted = format_hex_input(&e.value());
                                            new_cmd.set(formatted);
                                        } else {
                                            new_cmd.set(e.value());
                                        }
                                    },
                                }
                            }
                        }
                        label { class: "flex items-center gap-2 mt-2 ml-1 cursor-pointer",
                            input {
                                class: "w-4 h-4 rounded bg-[#0d0f10] border-[#2a2e33] checked:bg-primary checked:border-primary focus:ring-0 cursor-pointer accent-primary",
                                "type": "checkbox",
                                checked: "{new_hex}",
                                onchange: move |e| new_hex.set(e.value() == "true"),
                            }
                            span { class: "text-xs text-gray-400 font-bold select-none",
                                "Hex Mode"
                            }
                        }

                        div { class: "flex justify-end gap-2 mt-4",
                            button {
                                class: "px-3 py-1.5 text-xs text-gray-400 hover:text-white transition-colors",
                                onclick: move |_| show_form.set(false),
                                "Cancel"
                            }
                            button {
                                class: "px-3 py-1.5 text-xs bg-primary text-white rounded hover:bg-primary-hover shadow-lg shadow-primary/20 transition-all active:scale-95",
                                onclick: move |_| {
                                    if !new_label().is_empty() && !new_cmd().is_empty() {
                                        if new_hex() {
                                            if let Err(e) = parse_hex_string(&new_cmd()) {
                                                state.error(&format!("Macro Hex Error: {}", e));
                                                return;
                                            }
                                        }

                                        storage.write().add(new_label(), new_cmd(), new_hex());
                                        new_label.set(String::new());
                                        new_cmd.set(String::new());
                                        new_hex.set(false);
                                        show_form.set(false);
                                        state.success("Macro Added");
                                    } else {
                                        state.error("Please fill in all fields");
                                    }
                                },
                                "Add"
                            }
                        }
                    }
                    // Click outside to close
                    div {
                        class: "absolute inset-0 -z-10",
                        onclick: move |_| show_form.set(false),
                    }
                }
            }
        }
    }
}
