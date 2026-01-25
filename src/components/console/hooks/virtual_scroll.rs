use crate::components::console::hooks::data_request::use_data_request;
use crate::components::console::utils::layout_utils::{
    calculate_virtual_metrics, use_auto_scroller, use_window_resize,
};
use crate::config::{BOTTOM_BUFFER_EXTRA, LINE_HEIGHT, TOP_BUFFER};
use crate::state::AppState;
use crate::utils::calculate_window_size;
use dioxus::prelude::*;
use std::rc::Rc;

pub struct VirtualScroll {
    pub start_index: Signal<usize>,
    pub console_height: Signal<f64>,
    pub total_height: f64,
    pub offset_top: f64,
    pub console_handle: Signal<Option<Rc<MountedData>>>,
    pub sentinel_handle: Signal<Option<Rc<MountedData>>>,
    pub scale_factor: f64,
}

pub fn use_virtual_scroll() -> VirtualScroll {
    let state = use_context::<AppState>();

    let mut start_index = use_signal(|| 0usize);
    let console_height = use_signal(|| 600.0);

    let console_handle = use_signal(|| None::<Rc<MountedData>>);
    let sentinel_handle = use_signal(|| None::<Rc<MountedData>>);

    let total_lines = state.log.total_lines;

    let window_size = calculate_window_size(
        console_height(),
        LINE_HEIGHT,
        TOP_BUFFER + BOTTOM_BUFFER_EXTRA,
    );

    // Reset/Sync start index
    use_effect(move || {
        let total = total_lines();
        let start = start_index();

        if total == 0 {
            if start != 0 {
                start_index.set(0);
            }
            return;
        }

        if start >= total {
            if (state.ui.autoscroll)() {
                let page_size = (console_height() / LINE_HEIGHT).ceil() as usize;
                let new_start = total.saturating_sub(page_size);
                if start != new_start {
                    start_index.set(new_start);
                }
            } else if start != 0 {
                start_index.set(0);
            }
        }
    });

    use_window_resize(console_height, state.ui.autoscroll, sentinel_handle);
    use_data_request(start_index, window_size, total_lines);
    use_auto_scroller(state.ui.autoscroll, total_lines, sentinel_handle);

    let (total_height, scale_factor, offset_top) =
        calculate_virtual_metrics(total_lines(), start_index());

    VirtualScroll {
        start_index,
        console_height,
        total_height,
        offset_top,
        console_handle,
        sentinel_handle,
        scale_factor,
    }
}
