mod hooks;
mod toolbar;

use crate::state::AppState;
use crate::utils::terminal_bindings::Terminal;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::window;

pub use toolbar::TerminalToolbar;

pub struct AutoDisposeTerminal(pub Terminal);

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

#[component]
pub fn Xterm() -> Element {
    let mut terminal_div = use_signal(|| None::<web_sys::HtmlElement>);
    let term_instance = use_signal(|| None::<AutoDisposeTerminal>);
    let state = use_context::<AppState>();

    // Buffers for throttled operations
    let aggregation_buffer = Rc::new(RefCell::new(Vec::<u8>::new()));
    let send_buffer = Rc::new(RefCell::new(Vec::<u8>::new()));
    let resize_listener = use_signal(|| None::<gloo_events::EventListener>);

    // Clone for data ingestion effect
    let aggregation_buffer_for_effect = aggregation_buffer.clone();

    // Terminal setup effect
    use_effect(move || {
        if let Some(div) = terminal_div.read().as_ref() {
            if term_instance.read().is_some() {
                return;
            }
            hooks::setup_terminal(
                div,
                state,
                send_buffer.clone(),
                aggregation_buffer.clone(),
                term_instance,
                resize_listener,
            );
        }
    });

    // Option updates effect
    use_effect(move || {
        if let Some(term) = term_instance.read().as_ref() {
            let options = term.options();
            options.set_font_size(*state.ui.font_size.read());
            options.set_scrollback(*state.terminal.scrollback.read());
        }
    });

    // Data ingestion effect
    use_effect(move || {
        let _ = state.terminal.received_data.read();
        let data = state.terminal.take_data();
        if !data.is_empty() {
            aggregation_buffer_for_effect.borrow_mut().extend(data);
        }
    });

    rsx! {
        div { class: "relative w-full h-full flex flex-col",
            // Toolbar
            TerminalToolbar { term_instance }

            // Terminal Container
            div {
                class: "flex-1 w-full bg-black overflow-hidden",
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
}
