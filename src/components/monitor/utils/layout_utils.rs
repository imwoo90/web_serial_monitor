use crate::config::HEADER_OFFSET;
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
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        let mut update = move || {
            let window = match web_sys::window() {
                Some(w) => w,
                None => return,
            };

            if let Ok(Some(h)) = window.inner_height().map(|jv| jv.as_f64()) {
                let new_height = (h - HEADER_OFFSET).max(100.0);

                // Only update if height changed significantly
                // Using peek() here ensures this effect runs ONLY ONCE on mount
                if (*console_height.peek() - new_height).abs() > 0.1 {
                    console_height.set(new_height);
                }
            }
        };

        update(); // Initial execution
        let onresize = Closure::wrap(Box::new(update) as Box<dyn FnMut()>);
        window.set_onresize(Some(onresize.as_ref().unchecked_ref()));
        onresize.forget();
    });
    // Use use_resource to handle scrolling reactively.
    // This is more efficient as it automatically cancels previous tasks if a new change occurs
    // while we are waiting (TimeoutFuture).
    use_resource(move || async move {
        console_height(); // Subscribe to height changes
        let auto = autoscroll(); // Subscribe to autoscroll changes

        if auto {
            // Wait a tick for the DOM to update with new height
            gloo_timers::future::TimeoutFuture::new(10).await;
            if let Some(s) = sentinel.peek().as_ref() {
                let _ = s.scroll_to(ScrollBehavior::Instant).await;
            }
        }
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
/// Helper to calculate new scroll state (start_index and autoscroll)
/// Returns (new_start_index, should_autoscroll)
pub fn calculate_scroll_state(
    offset_y: f64,
    viewport_height: f64,
    total_lines: usize,
    scale_factor: f64,
    physical_total_height: f64,
    line_height: f64,
) -> (usize, bool) {
    use crate::config::TOP_BUFFER;
    use crate::utils::calculate_start_index;

    // 1. Physical Bottom Detection
    let is_at_bottom = if physical_total_height <= viewport_height {
        true
    } else {
        // Use a generous threshold (15px) to absorb browser or overscroll jitter
        offset_y + viewport_height >= physical_total_height - 15.0
    };

    // 2. Calculate Start Index with Clamping
    let new_index = if is_at_bottom {
        // If at bottom, force to the maximum possible start index
        let visible_lines = (viewport_height / line_height).ceil() as usize;
        total_lines.saturating_sub(visible_lines)
    } else {
        // Normal calculation: Convert physical to logical
        let logical_offset_y = offset_y * scale_factor;
        calculate_start_index(logical_offset_y, line_height, TOP_BUFFER)
    };

    (new_index, is_at_bottom)
}

/// Calculates virtual scroll metrics (total_height, offset_top, scale_factor)
pub fn calculate_virtual_metrics(
    total_lines: usize,
    start_index: usize,
    viewport_height: f64,
    line_height: f64,
) -> (f64, f64, f64) {
    use crate::config::{CONSOLE_BOTTOM_PADDING, CONSOLE_TOP_PADDING, VIRTUAL_SCROLL_THRESHOLD};

    let real_total_height =
        (total_lines as f64) * line_height + CONSOLE_TOP_PADDING + CONSOLE_BOTTOM_PADDING;

    // Use Dynamic x2 Scaling:
    // We pick a scale factor that is a power of 2 (1, 2, 4, 8...),
    // ensuring the physical scroll range stays under the threshold.
    let (total_height, scale_factor) = if real_total_height > VIRTUAL_SCROLL_THRESHOLD {
        let max_physical_scroll_range = VIRTUAL_SCROLL_THRESHOLD - viewport_height;
        let logical_scroll_range = real_total_height - viewport_height;

        // Find the smallest power of 2 such that logical_range / sf <= max_physical_range
        let ratio = logical_scroll_range / max_physical_scroll_range;
        let sf = 2.0_f64.powf(ratio.log2().ceil());

        let physical_total_height = (logical_scroll_range / sf) + viewport_height;
        (physical_total_height, sf)
    } else {
        (real_total_height, 1.0)
    };

    // Calculate logical offset top
    let logical_offset_top = (start_index as f64) * line_height;

    // Map logical offset to physical offset (Stable because scale_factor only changes at power-of-2 boundaries)
    // FLOOR the value to avoid sub-pixel rendering jitters
    let offset_top = (logical_offset_top / scale_factor).floor();

    (total_height, offset_top, scale_factor)
}
