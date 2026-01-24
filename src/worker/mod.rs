pub mod core;
pub mod processor;
pub mod storage;

use crate::components::console::types::WorkerMsg;
use crate::worker::processor::LogProcessor;
use crate::worker::storage::{get_opfs_root, init_opfs_session, new_session};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

/// Helper to detect the current app script path for worker spawning
pub fn get_app_script_path() -> String {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(scripts) = document.query_selector_all("script") {
                for i in 0..scripts.length() {
                    if let Some(item) = scripts.item(i) {
                        let script = item.unchecked_into::<web_sys::HtmlScriptElement>();
                        let src = script.src();
                        if !src.is_empty() && !src.contains("tailwindcss") && src.ends_with(".js") {
                            return src;
                        }
                    }
                }
            }
        }
    }
    // Fallback to a common default if detection fails
    "./serial_monitor.js".to_string()
}

pub fn process_worker_msg(
    msg: WorkerMsg,
    processor: &std::rc::Rc<std::cell::RefCell<LogProcessor>>,
    filename: &std::rc::Rc<std::cell::RefCell<Option<String>>>,
    root: &web_sys::FileSystemDirectoryHandle,
    scope: &web_sys::DedicatedWorkerGlobalScope,
) {
    let mut p = processor.borrow_mut();
    match msg {
        WorkerMsg::NewSession => {
            let filename_rc = filename.clone();
            let root_rc = root.clone();
            let scope_rc = scope.clone();
            let proc_for_spawn = processor.clone();
            spawn_local(async move {
                let lock_res = {
                    let mut f = filename_rc.borrow_mut();
                    match new_session(&root_rc, true, &mut f).await {
                        Ok(lock) => Ok(lock),
                        Err(e) => Err(e),
                    }
                };
                if let Ok(lock) = lock_res {
                    let mut pp = proc_for_spawn.borrow_mut();
                    let _ = pp.set_sync_handle(lock);
                    let _ = pp.clear();
                    let _ = scope_rc.post_message(
                        &serde_json::to_string(&WorkerMsg::TotalLines(0))
                            .unwrap()
                            .into(),
                    );
                }
            });
        }
        WorkerMsg::AppendLog(text) => {
            let _ = p.append_log(text);
        }
        WorkerMsg::AppendChunk { chunk, is_hex } => {
            let _ = p.append_chunk(&chunk, is_hex);
        }
        WorkerMsg::RequestWindow { start_line, count } => {
            if let Ok(val) = p.request_window(start_line, count) {
                if let Ok(lines) = serde_wasm_bindgen::from_value::<Vec<String>>(val) {
                    let resp = WorkerMsg::LogWindow { start_line, lines };
                    let _ = scope.post_message(&serde_json::to_string(&resp).unwrap().into());
                }
            }
        }
        WorkerMsg::Clear => {
            let _ = p.clear();
            let _ = scope.post_message(
                &serde_json::to_string(&WorkerMsg::TotalLines(0))
                    .unwrap()
                    .into(),
            );
        }
        WorkerMsg::SetLineEnding(mode) => {
            p.set_line_ending(&mode);
        }
        WorkerMsg::SearchLogs {
            query,
            match_case,
            use_regex,
            invert,
        } => {
            if let Ok(count) = p.search_logs(query, match_case, use_regex, invert) {
                let _ = scope.post_message(
                    &serde_json::to_string(&WorkerMsg::TotalLines(count as usize))
                        .unwrap()
                        .into(),
                );
            }
        }
        WorkerMsg::ExportLogs { include_timestamp } => {
            if let Ok(stream) = p.export_logs(include_timestamp) {
                let resp = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&resp, &"type".into(), &"EXPORT_STREAM".into());
                let _ = js_sys::Reflect::set(&resp, &"stream".into(), &stream);
                let transfer = js_sys::Array::of1(&stream);
                let _ = scope.post_message_with_transfer(&resp, &transfer);
            }
        }
        _ => {}
    }
}

#[wasm_bindgen]
pub fn start_worker() {
    let global = js_sys::global();
    let is_worker = js_sys::Reflect::has(&global, &"WorkerGlobalScope".into()).unwrap_or(false);

    if !is_worker {
        return;
    }

    spawn_local(async move {
        let mut processor = LogProcessor::new().expect("Failed to create LogProcessor");
        let mut current_filename: Option<String> = None;

        // Use the new init function from storage
        if let Ok(lock) = init_opfs_session(&mut current_filename).await {
            let _ = processor.set_sync_handle(lock);
        }

        let scope = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();
        let root = get_opfs_root().await.expect("Failed to get OPFS root");
        let mut last_count = 0;

        let processor_ptr = std::rc::Rc::new(std::cell::RefCell::new(processor));
        let filename_ptr = std::rc::Rc::new(std::cell::RefCell::new(current_filename));

        // Closures and clones for message handling
        let processor_for_msg = processor_ptr.clone();
        let filename_for_msg = filename_ptr.clone();
        let root_for_msg = root.clone();
        let scope_for_msg = scope.clone();
        let scope_for_loop = scope.clone();

        let onmessage = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            let data = event.data();

            // 1. Try generic JSON string (Control messages)
            if let Some(msg_str) = data.as_string() {
                if let Ok(msg) = serde_json::from_str::<WorkerMsg>(&msg_str) {
                    process_worker_msg(
                        msg,
                        &processor_for_msg,
                        &filename_for_msg,
                        &root_for_msg,
                        &scope_for_msg,
                    );
                }
                return;
            }

            // 2. Try Binary/Object optimized messages (Optimized AppendChunk)
            if data.is_object() {
                // Check if it's our optimized AppendChunk object
                // format: { cmd: "AppendChunk", chunk: Uint8Array, is_hex: bool }
                let cmd = js_sys::Reflect::get(&data, &"cmd".into())
                    .ok()
                    .and_then(|v| v.as_string());

                if let Some("AppendChunk") = cmd.as_deref() {
                    if let Ok(chunk_val) = js_sys::Reflect::get(&data, &"chunk".into()) {
                        let chunk = js_sys::Uint8Array::new(&chunk_val).to_vec();
                        let is_hex = js_sys::Reflect::get(&data, &"is_hex".into())
                            .ok()
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        let mut p = processor_for_msg.borrow_mut();
                        let _ = p.append_chunk(&chunk, is_hex);
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        // Interval for line count updates
        loop {
            gloo_timers::future::TimeoutFuture::new(50).await;
            let current = processor_ptr.borrow().get_line_count();
            if current != last_count {
                last_count = current;
                let _ = scope_for_loop.post_message(
                    &serde_json::to_string(&WorkerMsg::TotalLines(current as usize))
                        .unwrap()
                        .into(),
                );
            }
        }
    });
}
