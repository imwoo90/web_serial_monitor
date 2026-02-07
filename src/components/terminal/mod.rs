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

    // Data ingestion effect
    use_effect(move || {
        let _ = state.terminal.received_data.read();
        let data = state.terminal.take_data();
        if !data.is_empty() {
            aggregation_buffer_for_effect.borrow_mut().extend(data);
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
