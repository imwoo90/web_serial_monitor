use super::types::WorkerMsg;
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, Worker};

/// Hook to initialize Web Worker and handle messages
pub fn use_log_worker(
    mut total_lines: Signal<usize>,
    mut visible_logs: Signal<Vec<String>>,
    worker: Signal<Option<Worker>>,
) {
    use_effect(move || {
        if let Some(w) = worker() {
            let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
                let data = e.data();

                // Check for EXPORT_STREAM (Stream objects cannot be deserialized by serde)
                let type_key = JsValue::from_str("type");
                if let Ok(msg_type) = js_sys::Reflect::get(&data, &type_key) {
                    if msg_type == "EXPORT_STREAM" {
                        let stream_key = JsValue::from_str("stream");
                        if let Ok(stream) = js_sys::Reflect::get(&data, &stream_key) {
                            crate::utils::file_save::save_stream_to_disk(stream);
                            web_sys::console::log_1(&"Starting Download Stream...".into());
                            return;
                        }
                    }
                }

                if let Ok(msg) = serde_wasm_bindgen::from_value::<WorkerMsg>(data) {
                    match msg {
                        WorkerMsg::TotalLines(count) => {
                            total_lines.set(count);
                            if count == 0 {
                                visible_logs.set(Vec::new());
                            }
                        }
                        WorkerMsg::LogWindow { lines, .. } => visible_logs.set(lines),
                        _ => {}
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            w.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();
        }
    });
}

/// Hook to request a window of log data from Worker
pub fn use_data_request(
    start_index: Signal<usize>,
    window_size: usize,
    total_lines: Signal<usize>,
    worker: Signal<Option<Worker>>,
) {
    use_effect(move || {
        let start = start_index();
        total_lines(); // Also subscribe to changes in total line count
        if let Some(w) = worker.peek().as_ref() {
            let msg = WorkerMsg::RequestWindow {
                start_line: start,
                count: window_size,
            };
            if let Ok(js_obj) = serde_wasm_bindgen::to_value(&msg) {
                let _ = w.post_message(&js_obj);
            }
        }
    });
}
