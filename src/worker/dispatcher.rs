use crate::worker::types::WorkerMsg;
use crate::worker::WorkerState;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

pub fn handle_message(state_rc: Rc<RefCell<WorkerState>>, data: JsValue) {
    let mut state = state_rc.borrow_mut();

    if let Some(msg_str) = data.as_string() {
        if let Ok(msg) = serde_json::from_str::<WorkerMsg>(&msg_str) {
            match state.dispatch(msg) {
                Ok(false) => {
                    // NewSession needs async handling
                    drop(state);
                    WorkerState::handle_new_session(state_rc);
                }
                Ok(true) => {}
                Err(e) => {
                    state.send_error(e);
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
