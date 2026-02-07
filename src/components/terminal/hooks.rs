use crate::state::AppState;
use crate::utils::terminal_bindings::*;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::window;

/// Initialize the xterm.js terminal instance
pub fn setup_terminal(
    div: &web_sys::HtmlElement,
    state: AppState,
    send_buffer: Rc<RefCell<Vec<u8>>>,
    _aggregation_buffer: Rc<RefCell<Vec<u8>>>,
    mut term_instance: Signal<Option<super::AutoDisposeTerminal>>,
    mut resize_listener: Signal<Option<gloo_events::EventListener>>,
    mut fit_addon_signal: Signal<Option<XtermFitAddon>>,
) {
    let win = window().unwrap();
    let term_constructor = js_sys::Reflect::get(&win, &"Terminal".into()).unwrap();
    if term_constructor.is_undefined() {
        web_sys::console::error_1(&"xterm.js not loaded".into());
        return;
    }

    // Terminal options
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"convertEol".into(), &true.into()).unwrap();
    js_sys::Reflect::set(
        &options,
        &"theme".into(),
        &serde_wasm_bindgen::to_value(&serde_json::json!({
            "background": "#000000",
            "foreground": "#ffffff"
        }))
        .unwrap(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &options,
        &"fontSize".into(),
        &(*state.ui.font_size.read()).into(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &options,
        &"scrollback".into(),
        &(*state.terminal.scrollback.read()).into(),
    )
    .unwrap();

    let term = Terminal::new(&options);
    let fit_addon = XtermFitAddon::new_fit();
    term.load_addon(&fit_addon.clone().into());
    term.open(div);

    // Store fit_addon for external access
    fit_addon_signal.set(Some(fit_addon.clone().unchecked_into()));

    // Data sending closure (append to send_buffer for throttled sending)
    let send_buffer_for_input = send_buffer.clone();
    let on_data_closure = wasm_bindgen::prelude::Closure::wrap(Box::new(move |data: String| {
        let data_bytes = data.into_bytes();
        send_buffer_for_input.borrow_mut().extend(data_bytes);
    }) as Box<dyn FnMut(String)>);
    term.on_data(on_data_closure.as_ref().unchecked_ref());
    on_data_closure.forget();

    // Store terminal instance
    let term_for_instance: Terminal = term.clone().unchecked_into();
    term_instance.set(Some(super::AutoDisposeTerminal(term_for_instance)));

    // Initial fit
    let fit_initial: XtermFitAddon = fit_addon.clone().unchecked_into();
    spawn(async move {
        gloo_timers::future::TimeoutFuture::new(100).await;
        fit_initial.fit();
    });

    // Resize Handler
    let fit_for_resize = fit_addon;
    let listener = gloo_events::EventListener::new(&win, "resize", move |_| {
        let fit: XtermFitAddon = fit_for_resize.clone().unchecked_into();
        fit.fit();
    });
    resize_listener.set(Some(listener));

    // Scroll Handler (Update Autoscroll State)
    let term_for_scroll: Terminal = term.clone().unchecked_into();
    let mut autoscroll_signal = state.terminal.autoscroll;
    let last_update = Rc::new(RefCell::new(0.0));

    let on_scroll_closure = wasm_bindgen::prelude::Closure::wrap(Box::new(move |_| {
        let now = js_sys::Date::now();
        let mut last = last_update.borrow_mut();

        // Throttle updates to max once per 100ms to prevent freezing during high-speed logs
        if now - *last < 100.0 {
            return;
        }
        *last = now;

        let buffer = term_for_scroll.buffer().active();
        // Allow 1 line tolerance
        let is_at_bottom = (buffer.base_y() - buffer.viewport_y()).abs() <= 1;

        if *autoscroll_signal.peek() != is_at_bottom {
            *autoscroll_signal.write() = is_at_bottom;
        }
    }) as Box<dyn FnMut(i32)>);
    term.on_scroll(on_scroll_closure.as_ref().unchecked_ref());
    on_scroll_closure.forget();
}
