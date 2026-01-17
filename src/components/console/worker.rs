use super::types::WorkerMsg;
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, Worker};

/// Web Worker리를 초기화하고 메시지를 처리하는 훅
pub fn use_log_worker(
    mut total_lines: Signal<usize>,
    mut visible_logs: Signal<Vec<String>>,
    mut worker: Signal<Option<Worker>>,
) {
    use_effect(move || {
        let worker_path = asset!("/assets/log_worker.js").to_string();
        let w = Worker::new(&worker_path).expect("Failed to create worker");

        let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(msg) = serde_wasm_bindgen::from_value::<WorkerMsg>(e.data()) {
                match msg {
                    WorkerMsg::TotalLines(count) => total_lines.set(count),
                    WorkerMsg::LogWindow { lines, .. } => visible_logs.set(lines),
                    _ => {}
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        w.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
        worker.set(Some(w));
    });
}

/// Worker에 로그 데이터 윈도우를 요청하는 훅
pub fn use_data_request(
    start_index: Signal<usize>,
    window_size: usize,
    total_lines: Signal<usize>,
    worker: Signal<Option<Worker>>,
) {
    use_effect(move || {
        let start = start_index();
        total_lines(); // 전체 라인 수 변화도 구독
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
