mod hooks;
mod toolbar;

use crate::components::ui::buttons::ResumeScrollButton;
use crate::components::ui::console::ConsoleFrame;
use crate::state::AppState;
use crate::utils::terminal_bindings::{Terminal, XtermFitAddon};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::window;

pub use toolbar::TerminalToolbar;

pub struct AutoDisposeTerminal(pub Terminal);

#[component]
pub fn TerminalView(term_instance: Signal<Option<AutoDisposeTerminal>>) -> Element {
    let app_state = use_context::<AppState>();

    rsx! {
        ConsoleFrame {
            // Toolbar
            TerminalToolbar { term_instance }

            // Terminal Content
            Xterm { term_instance }

            // Resume Scroll Button
            if !*app_state.terminal.autoscroll.read() {
                ResumeScrollButton {
                    onclick: move |_| {
                        if let Some(term) = term_instance.read().as_ref() {
                            term.scroll_to_bottom();
                        }
                    },
                }
            }
        }
    }
}

impl Drop for AutoDisposeTerminal {
    fn drop(&mut self) {
        self.0.dispose();
    }
}

impl std::ops::Deref for AutoDisposeTerminal {
    type Target = Terminal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct XtermProps {
    term_instance: Signal<Option<AutoDisposeTerminal>>,
}

#[component]
pub fn Xterm(props: XtermProps) -> Element {
    let mut terminal_div = use_signal(|| None::<web_sys::HtmlElement>);
    let term_instance = props.term_instance;
    let fit_addon = use_signal(|| None::<XtermFitAddon>);
    let state = use_context::<AppState>();

    // Buffers for throttled operations - persisted across renders
    let aggregation_buffer = use_signal(|| Rc::new(RefCell::new(Vec::<u8>::new())));
    let send_buffer = use_signal(|| Rc::new(RefCell::new(Vec::<u8>::new())));
    let resize_listener = use_signal(|| None::<gloo_events::EventListener>);

    // Terminal setup effect
    use_effect(move || {
        if let Some(div) = terminal_div.read().as_ref() {
            if term_instance.read().is_some() {
                return;
            }
            hooks::setup_terminal(
                div,
                state,
                send_buffer(),
                aggregation_buffer(),
                term_instance,
                resize_listener,
                fit_addon,
            );
        }
    });

    // Option updates effect - also call fit() when font size changes
    use_effect(move || {
        let font_size = *state.ui.font_size.read();
        let scrollback = *state.terminal.scrollback.read();

        if let Some(term) = term_instance.read().as_ref() {
            let options = term.options();
            options.set_font_size(font_size);
            options.set_scrollback(scrollback);

            // Call fit() after font size change to recalculate rows/cols
            if let Some(fit) = fit_addon.read().as_ref() {
                fit.fit();
            }
        }
    });

    // Data ingestion loop
    use_effect(move || {
        let _ = state.terminal.received_data.read();
        let data = state.terminal.take_data();
        if !data.is_empty() {
            aggregation_buffer.read().borrow_mut().extend(data);
        }
    });

    // Terminal write loop (100ms)
    use_resource(move || {
        let mut lines_signal = state.terminal.lines;
        async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(100).await;
                if let Some(term) = term_instance.read().as_ref() {
                    let buffer_rc = aggregation_buffer.read().clone();
                    let mut data_vec = buffer_rc.borrow_mut();
                    if !data_vec.is_empty() {
                        let chunk = std::mem::take(&mut *data_vec);
                        drop(data_vec);
                        let array = js_sys::Uint8Array::from(chunk.as_slice());
                        term.write_chunk(&array);

                        // Update line count
                        let lines = term.buffer().active().length();
                        *lines_signal.write() = lines as usize;
                    }
                }
            }
        }
    });

    // Serial send loop (60Hz)
    use_resource(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(16).await;
            let data = {
                let buffer_rc = send_buffer.read().clone();
                let mut buf = buffer_rc.borrow_mut();
                if buf.is_empty() {
                    continue;
                }
                std::mem::take(&mut *buf)
            };
            if let Some(port) = state.conn.port.peek().clone() {
                if let Err(e) = crate::utils::serial_api::send_data(&port, &data).await {
                    web_sys::console::error_1(&e);
                }
            }
        }
    });

    rsx! {
        // Terminal Container
        div {
            class: "flex-1 w-full bg-transparent overflow-hidden pl-2",
            id: "xterm-container",
            onmounted: move |_| {
                if let Some(element) = window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .get_element_by_id("xterm-container")
                {
                    if let Ok(html_elem) = element.dyn_into::<web_sys::HtmlElement>() {
                        terminal_div.set(Some(html_elem));
                    }
                }
            },
        }
    }
}
