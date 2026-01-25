use crate::state::AppState;
use crate::utils::{send_chunk_to_worker, send_worker_msg};
use crate::worker::types::WorkerMsg;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct WorkerBridge {
    worker_sig: Signal<Option<web_sys::Worker>>,
}

impl WorkerBridge {
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

    pub fn set_line_ending(&self, ending: String) {
        self.send(WorkerMsg::SetLineEnding(ending));
    }

    pub fn append_log(&self, text: String) {
        self.send(WorkerMsg::AppendLog(text));
    }

    pub fn append_chunk(&self, chunk: &[u8], is_hex: bool) {
        if let Some(w) = self.worker_sig.read().as_ref() {
            send_chunk_to_worker(w, chunk, is_hex);
        }
    }

    pub fn request_window(&self, start_line: usize, count: usize) {
        self.send(WorkerMsg::RequestWindow { start_line, count });
    }

    pub fn new_session(&self) {
        self.send(WorkerMsg::NewSession);
    }
}

pub fn use_worker_bridge() -> WorkerBridge {
    let state = use_context::<AppState>();
    WorkerBridge::new(state.log_worker)
}
