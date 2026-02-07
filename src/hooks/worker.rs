use crate::state::AppState;
use crate::types::WorkerMsg;
use crate::utils::{send_chunk_to_worker, send_worker_msg};
use dioxus::prelude::*;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy)]
pub struct WorkerController {
    worker_sig: Signal<Option<web_sys::Worker>>,
}

impl WorkerController {
    pub fn new(worker_sig: Signal<Option<web_sys::Worker>>) -> Self {
        Self { worker_sig }
    }

    fn send(&self, msg: WorkerMsg) {
        if let Some(w) = self.worker_sig.read().as_ref() {
            send_worker_msg(w, msg);
        }
    }

    pub fn clear(&self) {
        self.send(WorkerMsg::Clear);
    }

    pub fn search(&self, query: String, match_case: bool, use_regex: bool, invert: bool) {
        self.send(WorkerMsg::SearchLogs {
            query,
            match_case,
            use_regex,
            invert,
        });
    }

    pub fn export(&self, include_timestamp: bool) {
        self.send(WorkerMsg::ExportLogs { include_timestamp });
    }

    pub fn append_chunk(&self, chunk: js_sys::Uint8Array, is_hex: bool) {
        if let Some(w) = self.worker_sig.read().as_ref() {
            send_chunk_to_worker(w, chunk, is_hex);
        }
    }

    pub fn set_timestamp_state(&self, enabled: bool) {
        self.send(WorkerMsg::SetTimestampState(enabled));
    }

    pub fn request_window(&self, start_line: usize, count: usize) {
        self.send(WorkerMsg::RequestWindow { start_line, count });
    }

    pub fn new_session(&self) {
        self.send(WorkerMsg::NewSession);
    }

    pub fn set_mode(&self, mode: crate::state::ViewMode) {
        self.send(WorkerMsg::SetMode(mode));
    }
}

pub fn use_worker_controller() -> WorkerController {
    let mut state = use_context::<AppState>();

    // Worker Initialization
    use_effect(move || {
        if state.conn.log_worker.read().is_none() {
            let script_path = crate::worker::get_app_script_path();
            let options = web_sys::WorkerOptions::new();
            options.set_type(web_sys::WorkerType::Module);

            if let Ok(worker) = web_sys::Worker::new_with_options(&script_path, &options) {
                setup_worker_message_handler(&worker, state);
                state.conn.log_worker.set(Some(worker));
            }
        }
    });

    WorkerController::new(state.conn.log_worker)
}

fn setup_worker_message_handler(worker: &web_sys::Worker, state: AppState) {
    let mut tl = state.log.total_lines;
    let mut vl = state.log.visible_logs;

    let callback = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
        let data = event.data();

        // 1. Handle Binary/Object Messages (Stream Exports)
        if let Ok(obj) = data.clone().dyn_into::<js_sys::Object>() {
            if let Ok(msg_type) = js_sys::Reflect::get(&obj, &"type".into()) {
                if msg_type.as_string() == Some("EXPORT_STREAM".to_string()) {
                    if let Ok(stream) = js_sys::Reflect::get(&obj, &"stream".into()) {
                        crate::utils::file_save::save_stream_to_disk(stream);
                        return;
                    }
                }
            }
        }

        // 2. Handle Structured Messages (WorkerMsg)
        if let Some(msg_str) = data.as_string() {
            if let Ok(msg) = serde_json::from_str::<WorkerMsg>(&msg_str) {
                match msg {
                    WorkerMsg::TotalLines(count) => {
                        tl.set(count);
                        if count == 0 {
                            vl.set(Vec::new());
                        }
                    }
                    WorkerMsg::LogWindow { lines, .. } => {
                        vl.set(lines);
                    }
                    WorkerMsg::Error(msg) => {
                        state.error(&format!("Worker Error: {}", msg));
                    }
                    WorkerMsg::ActiveLine(line) => {
                        { state.log.active_line }.set(line);
                    }
                    _ => {}
                }
            }
        }
    }) as Box<dyn FnMut(_)>);

    worker.set_onmessage(Some(callback.as_ref().unchecked_ref()));
    callback.forget();
}
