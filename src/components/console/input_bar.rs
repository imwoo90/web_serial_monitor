use crate::components::common::ToggleSwitch;
use crate::serial;
use crate::state::{AppState, LineEnding};
use dioxus::prelude::*;

#[component]
fn LineEndSelector(
    label: &'static str,
    selected: LineEnding,
    onselect: EventHandler<LineEnding>,
    active_class: &'static str,
    is_rx: bool,
) -> Element {
    rsx! {
        div { class: "flex items-center gap-2",
            span { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest", "{label}" }
            div { class: "flex bg-[#0d0f10] p-0.5 rounded-lg border border-[#2a2e33]",
                for ending in [LineEnding::None, LineEnding::NL, LineEnding::CR, LineEnding::NLCR] {
                    button {
                        class: "px-2 py-1 rounded text-[10px] font-bold transition-all duration-200",
                        class: if selected == ending { "{active_class} border shadow-sm" } else { "text-gray-500 hover:text-white" },
                        onclick: move |_| onselect.call(ending),
                        match ending {
                            LineEnding::None => if is_rx { "RAW" } else { "NONE" },
                            LineEnding::NL => "LF",
                            LineEnding::CR => "CR",
                            LineEnding::NLCR => "CRLF",
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn InputBar() -> Element {
    let mut state = use_context::<AppState>();
    let mut input_value = use_signal(String::new);

    let rx_ending = (state.rx_line_ending)();
    let tx_ending = (state.line_ending)();

    let on_send = move || {
        spawn(async move {
            let text = input_value();
            if text.is_empty() {
                return;
            }

            let mut data = text.clone().into_bytes();
            match (state.line_ending)() {
                LineEnding::NL => data.push(b'\n'),
                LineEnding::CR => data.push(b'\r'),
                LineEnding::NLCR => {
                    data.push(b'\r');
                    data.push(b'\n');
                }
                _ => {}
            }

            if let Some(wrapper) = (state.port)() {
                if serial::send_data(&wrapper.0, &data).await.is_ok() {
                    // Success
                    input_value.set(String::new());
                } else {
                    // Fail
                }
            } else {
                // Not connected
            }
        });
    };

    rsx! {
        div { class: "shrink-0 p-5 pt-3 bg-background-dark border-t border-[#2a2e33] z-20 relative",
            div { class: "absolute top-0 left-0 right-0 h-px bg-linear-to-r from-transparent via-primary/20 to-transparent" }
            div { class: "flex flex-col gap-3",
                div { class: "flex items-center justify-between",
                    // RX Controls Group
                    div { class: "flex items-center gap-4",
                        LineEndSelector {
                            label: "RX Parse",
                            selected: rx_ending,
                            onselect: move |val| state.rx_line_ending.set(val),
                            active_class: "bg-emerald-500/20 text-emerald-500 border-emerald-500/20",
                            is_rx: true,
                        }
                        div { class: "w-px h-6 bg-[#2a2e33]" }
                        ToggleSwitch {
                            label: "HEX VIEW",
                            active: (state.is_hex_view)(),
                            onclick: move |_| state.is_hex_view.set(!(state.is_hex_view)()),
                        }
                    }
                    // TX Controls
                    LineEndSelector {
                        label: "Payload",
                        selected: tx_ending,
                        onselect: move |val| state.line_ending.set(val),
                        active_class: "bg-primary/20 text-primary border-primary/20",
                        is_rx: false,
                    }
                }
                div { class: "flex gap-3 items-stretch h-12",
                    div { class: "relative flex-1",
                        input {
                            class: "w-full h-full bg-[#0d0f10] text-sm text-white placeholder-gray-600 px-4 rounded-xl border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none shadow-inset-input transition-all",
                            placeholder: "Enter ASCII command...",
                            "type": "text",
                            value: "{input_value}",
                            oninput: move |evt| input_value.set(evt.value()),
                            onkeydown: move |evt| {
                                if evt.key() == Key::Enter {
                                    on_send();
                                }
                            }
                        }
                        div { class: "absolute right-3 top-1/2 -translate-y-1/2 text-gray-600 pointer-events-none",
                            span { class: "material-symbols-outlined text-[16px]", "keyboard" }
                        }
                    }
                    button {
                        class: "h-full aspect-square bg-primary text-surface rounded-xl flex items-center justify-center hover:bg-white transition-all hover:shadow-[0_0_15px_rgba(255,255,255,0.4)] active:scale-95 group",
                        onclick: move |_| on_send(),
                        span { class: "material-symbols-outlined text-[22px] group-hover:rotate-45 transition-transform duration-300",
                            "send"
                        }
                    }
                }
            }
        }
    }
}
