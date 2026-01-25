use crate::worker::processor::LogProcessor;
use crate::worker::storage::{get_opfs_root, init_opfs_session};
use crate::worker::types::WorkerMsg;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use wasm_bindgen_futures::spawn_local;

pub mod dispatcher;
pub mod error;
pub mod formatter;
pub mod index;
pub mod processor;
pub mod search;
pub mod storage;
pub mod types;

pub(crate) struct WorkerState {
    pub(crate) proc: LogProcessor,
    pub(crate) filename: Option<String>,
    pub(crate) root: web_sys::FileSystemDirectoryHandle,
    pub(crate) scope: web_sys::DedicatedWorkerGlobalScope,
}

impl WorkerState {
    async fn new() -> Result<Self, JsValue> {
        let mut proc = LogProcessor::new()?;
        let mut filename: Option<String> = None;
        if let Ok(lock) = init_opfs_session(&mut filename).await {
            let _ = proc.set_sync_handle(lock);
        }

        let scope = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();
        let root = get_opfs_root().await?;

        Ok(Self {
            proc,
            filename,
            root,
            scope,
        })
    }

    pub(crate) fn handle_new_session(state_rc: Rc<RefCell<Self>>) {
        spawn_local(async move {
            let (root, filename_opt) = {
                let s = state_rc.borrow();
                (s.root.clone(), s.filename.clone())
            };
            let mut filename = filename_opt;
            if let Ok(lock) = crate::worker::storage::new_session(&root, true, &mut filename).await
            {
                let mut s = state_rc.borrow_mut();
                s.filename = filename;
                let _ = s.proc.set_sync_handle(lock);
                let _ = s.proc.clear();
                s.send_msg(WorkerMsg::TotalLines(0));
            }
        });
    }

    pub(crate) fn dispatch(&mut self, msg: WorkerMsg) -> Result<bool, JsValue> {
        match msg {
            WorkerMsg::NewSession => return Ok(false),
            WorkerMsg::AppendChunk { chunk, is_hex } => {
                self.proc.append_chunk(&chunk, is_hex)?;
            }
            WorkerMsg::AppendLog(text) => {
                self.proc.append_log(text)?;
            }
            WorkerMsg::RequestWindow { start_line, count } => {
                let val = self.proc.request_window(start_line, count)?;
                let lines = serde_wasm_bindgen::from_value::<Vec<String>>(val)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                self.send_msg(WorkerMsg::LogWindow { start_line, lines });
            }
            WorkerMsg::Clear => {
                self.proc.clear()?;
                self.send_msg(WorkerMsg::TotalLines(0));
            }
            WorkerMsg::SetLineEnding(mode) => self.proc.set_line_ending(&mode),
            WorkerMsg::SearchLogs {
                query,
                match_case,
                use_regex,
                invert,
            } => {
                let count = self
                    .proc
                    .search_logs(query, match_case, use_regex, invert)?;
                self.send_msg(WorkerMsg::TotalLines(count as usize));
            }
            WorkerMsg::ExportLogs { include_timestamp } => {
                let stream = self.proc.export_logs(include_timestamp)?;
                let resp = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&resp, &"type".into(), &"EXPORT_STREAM".into());
                let _ = js_sys::Reflect::set(&resp, &"stream".into(), &stream);
                let _ = self
                    .scope
                    .post_message_with_transfer(&resp, &js_sys::Array::of1(&stream));
            }
            _ => {}
        }
        Ok(true)
    }

    pub(crate) fn send_msg(&self, msg: WorkerMsg) {
        if let Ok(s) = serde_json::to_string(&msg) {
            let _ = self.scope.post_message(&s.into());
        }
    }

    pub(crate) fn send_error(&self, err: JsValue) {
        let msg = format!("{:?}", err);
        self.send_msg(WorkerMsg::Error(msg));
    }
}

#[wasm_bindgen]
pub fn start_worker() {
    if !js_sys::Reflect::has(&js_sys::global(), &"WorkerGlobalScope".into()).unwrap_or(false) {
        return;
    }

    spawn_local(async move {
        let state = match WorkerState::new().await {
            Ok(s) => Rc::new(RefCell::new(s)),
            Err(e) => {
                web_sys::console::error_1(&e);
                return;
            }
        };

        let onmessage = {
            let s_ptr = state.clone();
            Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
                dispatcher::handle_message(s_ptr.clone(), event.data());
            }) as Box<dyn FnMut(_)>)
        };

        let scope = state.borrow().scope.clone();
        scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let mut last_count = 0;
        loop {
            gloo_timers::future::TimeoutFuture::new(crate::config::WORKER_STATUS_INTERVAL_MS).await;
            let current = state.borrow().proc.get_line_count();
            if current != last_count {
                last_count = current;
                state
                    .borrow()
                    .send_msg(WorkerMsg::TotalLines(current as usize));
            }
        }
    });
}

pub fn get_app_script_path() -> String {
    let window = web_sys::window().expect("no global window instance found");
    let document = window.document().expect("should have a document on window");
    if let Ok(scripts) = document.query_selector_all("script[type='module']") {
        for i in 0..scripts.length() {
            if let Some(node) = scripts.item(i) {
                let script: web_sys::HtmlScriptElement = node.unchecked_into();
                let src = script.src();
                let s = src.to_lowercase();
                if (s.contains("serial_monitor") || s.contains("web_serial_monitor"))
                    && !s.contains("snippets")
                    && s.ends_with(".js")
                {
                    return src;
                }
            }
        }
    }
    "./serial_monitor.js".into()
}
