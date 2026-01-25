use crate::worker::processor::LogProcessor;
use crate::worker::storage::{get_opfs_root, init_opfs_session, new_session};
use crate::worker::types::WorkerMsg;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// Worker state that manages the log processor and OPFS session
pub(crate) struct WorkerState {
    pub(crate) proc: LogProcessor,
    pub(crate) filename: Option<String>,
    pub(crate) root: web_sys::FileSystemDirectoryHandle,
    pub(crate) scope: web_sys::DedicatedWorkerGlobalScope,
}

impl WorkerState {
    /// Creates a new WorkerState instance
    pub(crate) async fn new() -> Result<Self, JsValue> {
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

    /// Handles creating a new session asynchronously
    pub(crate) fn handle_new_session(state_rc: Rc<RefCell<Self>>) {
        spawn_local(async move {
            let (root, filename_opt) = {
                let s = state_rc.borrow();
                (s.root.clone(), s.filename.clone())
            };
            let mut filename = filename_opt;
            if let Ok(lock) = new_session(&root, true, &mut filename).await {
                let mut s = state_rc.borrow_mut();
                s.filename = filename;
                let _ = s.proc.set_sync_handle(lock);
                let _ = s.proc.clear();
                s.send_msg(WorkerMsg::TotalLines(0));
            }
        });
    }

    /// Sends a message to the main thread
    pub(crate) fn send_msg(&self, msg: WorkerMsg) {
        if let Ok(s) = serde_json::to_string(&msg) {
            let _ = self.scope.post_message(&s.into());
        }
    }

    /// Sends an error message to the main thread
    pub(crate) fn send_error(&self, err: JsValue) {
        let msg = format!("{:?}", err);
        self.send_msg(WorkerMsg::Error(msg));
    }
}
