use crate::components::console::hooks::data_request::use_data_request;
use crate::components::console::utils::layout_utils::{
    calculate_scroll_state, calculate_virtual_metrics, use_auto_scroller, use_window_resize,
};
use crate::config::{BOTTOM_BUFFER_EXTRA, LINE_HEIGHT, TOP_BUFFER};
use crate::state::AppState;
use crate::utils::calculate_window_size;
use dioxus::prelude::*;
use std::rc::Rc;

pub struct VirtualScroll {
    pub start_index: Signal<usize>,
    pub _console_height: Signal<f64>,
    pub total_height: f64,
    pub offset_top: f64,
    pub _scale_factor: f64,
    pub console_handle: Signal<Option<Rc<MountedData>>>,
    pub sentinel_handle: Signal<Option<Rc<MountedData>>>,
    pub scroll_task: Resource<()>,
    pub _height_task: Resource<()>,
}

pub fn use_virtual_scroll() -> VirtualScroll {
    let state = use_context::<AppState>();

    let mut start_index = use_signal(|| 0usize);
    let mut console_height = use_signal(|| 600.0);

    let console_handle = use_signal(|| None::<Rc<MountedData>>);
    let sentinel_handle = use_signal(|| None::<Rc<MountedData>>);

    let total_lines = state.log.total_lines;

    let window_size = calculate_window_size(
        console_height(),
        LINE_HEIGHT,
        TOP_BUFFER + BOTTOM_BUFFER_EXTRA,
    );

    use_window_resize(console_height, state.ui.autoscroll, sentinel_handle);
    use_data_request(start_index, window_size, total_lines);
    use_auto_scroller(state.ui.autoscroll, total_lines, sentinel_handle);

    let (total_height, offset_top, scale_factor) =
        calculate_virtual_metrics(total_lines(), start_index(), console_height());

    // Height update task
    let height_task = use_resource(move || {
        let handle = (console_handle)();
        async move {
            if let Some(handle) = handle {
                if let Ok(rect) = handle.get_client_rect().await {
                    console_height.set(rect.height());
                }
            }
        }
    });

    // Scroll task
    let scroll_task = use_resource(move || {
        let handle = console_handle.peek().as_ref().cloned();
        let total_lines = (state.log.total_lines)();
        let current_height = *console_height.read();
        let current_total_height = total_height;
        let current_scale = scale_factor;
        async move {
            if let Some(handle) = handle {
                if let Ok(offset) = handle.get_scroll_offset().await {
                    let (new_index, is_at_bottom) = calculate_scroll_state(
                        offset.y,
                        current_height,
                        total_lines,
                        current_scale,
                        current_total_height,
                    );
                    if (start_index)() != new_index {
                        start_index.set(new_index);
                    }
                    if (state.ui.autoscroll)() != is_at_bottom {
                        state.ui.set_autoscroll(is_at_bottom);
                    }
                }
            }
        }
    });

    VirtualScroll {
        start_index,
        _console_height: console_height,
        total_height,
        offset_top,
        _scale_factor: scale_factor,
        console_handle,
        sentinel_handle,
        scroll_task,
        _height_task: height_task,
    }
}
