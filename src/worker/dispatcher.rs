use crate::worker::storage::new_session;
use crate::worker::types::WorkerMsg;
use crate::worker::WorkerState;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use wasm_bindgen_futures::spawn_local;

pub fn handle_message(state_rc: Rc<RefCell<WorkerState>>, data: JsValue) {
    let mut state = state_rc.borrow_mut();

    if let Some(msg_str) = data.as_string() {
        if let Ok(msg) = serde_json::from_str::<WorkerMsg>(&msg_str) {
            match msg {
                WorkerMsg::NewSession => {
                    // Special handling for async NewSession to avoid double borrow
                    drop(state); // release borrow before async call
                    let s_ptr_inner = state_rc.clone();
                    spawn_local(async move {
                        let (root, mut filename) = {
                            let s = s_ptr_inner.borrow();
                            (s.root.clone(), s.filename.clone())
                        };
                        if let Ok(lock) = new_session(&root, true, &mut filename).await {
                            let mut s = s_ptr_inner.borrow_mut();
                            s.filename = filename;
                            let _ = s.proc.set_sync_handle(lock);
                            let _ = s.proc.clear();
                            s.send_msg(WorkerMsg::TotalLines(0));
                        }
                    });
                }
                _ => {
                    if let Err(e) = state.dispatch(msg) {
                        state.send_error(e);
                    }
                }
            }
        }
    } else if data.is_object() {
        // Optimized path for binary chunks
        if let Err(e) = handle_object_message(&mut state, &data) {
            state.send_error(e);
        }
    }
}

fn handle_object_message(state: &mut WorkerState, data: &JsValue) -> Result<(), JsValue> {
    let cmd = js_sys::Reflect::get(data, &"cmd".into())
        .ok()
        .and_then(|v| v.as_string());

    match cmd.as_deref() {
        Some("AppendChunk") => {
            if let Ok(chunk_val) = js_sys::Reflect::get(data, &"chunk".into()) {
                let chunk = js_sys::Uint8Array::new(&chunk_val).to_vec();
                let is_hex = js_sys::Reflect::get(data, &"is_hex".into())
                    .ok()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                state.proc.append_chunk(&chunk, is_hex)?;
            }
        }
        _ => {}
    }
    Ok(())
}
