use super::constants::HEADER_OFFSET;
use dioxus::prelude::*;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// Hook to handle window resize events and adjust console height
pub fn use_window_resize(
    mut console_height: Signal<f64>,
    autoscroll: Signal<bool>,
    sentinel: Signal<Option<Rc<MountedData>>>,
) {
    use_effect(move || {
        let mut update = move || {
            if let Ok(Some(h)) = web_sys::window()
                .unwrap()
                .inner_height()
                .map(|jv| jv.as_f64())
            {
                console_height.set((h - HEADER_OFFSET).max(100.0));
                // Force scroll to bottom during resize if autoscroll is enabled
                if (autoscroll)() {
                    if let Some(s) = sentinel.peek().as_ref() {
                        let _ = s.scroll_to(ScrollBehavior::Instant);
                    }
                }
            }
        };
        update(); // Initial execution
        let onresize = Closure::wrap(Box::new(update) as Box<dyn FnMut()>);
        web_sys::window()
            .unwrap()
            .set_onresize(Some(onresize.as_ref().unchecked_ref()));
        onresize.forget();
    });
}

/// Hook to manage auto-scroll functionality
pub fn use_auto_scroller(
    autoscroll: Signal<bool>,
    total_lines: Signal<usize>,
    _sentinel: Signal<Option<Rc<MountedData>>>, // Sentinel no longer needed
) {
    use_effect(move || {
        total_lines(); // React to changes
        if (autoscroll)() {
            // Use plain JS to set scrollTop ONLY, preserving scrollLeft.
            // Dioxus visible/scrollTo APIs often mess with X-axis.
            // Element ID is "console-output"
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(el) = document.get_element_by_id("console-output") {
                        // scrollTop = scrollHeight
                        let scroll_height = el.scroll_height();
                        el.set_scroll_top(scroll_height);
                    }
                }
            }
        }
    });
}

/// Helper to calculate new scroll state (start_index and autoscroll)
/// Returns (new_start_index, should_autoscroll)
pub fn calculate_scroll_state(
    offset_y: f64,
    viewport_height: f64,
    total_lines: usize,
) -> (usize, bool) {
    use super::constants::{CONSOLE_BOTTOM_PADDING, CONSOLE_TOP_PADDING, LINE_HEIGHT, TOP_BUFFER};
    use crate::utils::calculate_start_index;

    // 1. Calculate Virtual Scroll Index
    let new_index = calculate_start_index(offset_y, LINE_HEIGHT, TOP_BUFFER);

    // 2. Autoscroll Detection (Math-based)
    let content_height =
        (total_lines as f64) * LINE_HEIGHT + CONSOLE_TOP_PADDING + CONSOLE_BOTTOM_PADDING;

    // Allow small buffer (e.g. 10px) for precision
    let is_at_bottom = if content_height <= viewport_height {
        true
    } else {
        offset_y + viewport_height >= content_height - 10.0
    };

    (new_index, is_at_bottom)
}

// Removed ConsoleHeader and ResumeScrollButton to separate files
