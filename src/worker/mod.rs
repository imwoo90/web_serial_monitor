use crate::components::console::types::WorkerMsg;
use crate::worker::processor::LogProcessor;
use crate::worker::storage::{get_opfs_root, init_opfs_session, new_session};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

pub mod processor;
pub mod storage;

#[wasm_bindgen]
pub fn start_worker() {
    if !js_sys::Reflect::has(&js_sys::global(), &"WorkerGlobalScope".into()).unwrap_or(false) {
        return;
    }

    spawn_local(async move {
        let mut proc = LogProcessor::new().expect("Failed to create LogProcessor");
        let mut filename: Option<String> = None;
        if let Ok(lock) = init_opfs_session(&mut filename).await {
            let _ = proc.set_sync_handle(lock);
        }

        let scope = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();
        let root = get_opfs_root().await.expect("Failed to get OPFS root");
        let (proc_ptr, file_ptr) = (
            std::rc::Rc::new(std::cell::RefCell::new(proc)),
            std::rc::Rc::new(std::cell::RefCell::new(filename)),
        );

        let onmessage = {
            let (p, f, r, s) = (
                proc_ptr.clone(),
                file_ptr.clone(),
                root.clone(),
                scope.clone(),
            );
            Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
                let data = event.data();
                if let Some(msg_str) = data.as_string() {
                    if let Ok(msg) = serde_json::from_str::<WorkerMsg>(&msg_str) {
                        dispatch_msg(msg, &p, &f, &r, &s);
                    }
                } else if data.is_object() {
                    let cmd = js_sys::Reflect::get(&data, &"cmd".into())
                        .ok()
                        .and_then(|v| v.as_string());
                    if cmd.as_deref() == Some("AppendChunk") {
                        if let Ok(chunk_val) = js_sys::Reflect::get(&data, &"chunk".into()) {
                            let chunk = js_sys::Uint8Array::new(&chunk_val).to_vec();
                            let is_hex = js_sys::Reflect::get(&data, &"is_hex".into())
                                .ok()
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            let _ = p.borrow_mut().append_chunk(&chunk, is_hex);
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>)
        };
        scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let mut last_count = 0;
        loop {
            gloo_timers::future::TimeoutFuture::new(50).await;
            let current = proc_ptr.borrow().get_line_count();
            if current != last_count {
                last_count = current;
                let _ = scope.post_message(
                    &serde_json::to_string(&WorkerMsg::TotalLines(current as usize))
                        .unwrap()
                        .into(),
                );
            }
        }
    });
}

fn dispatch_msg(
    msg: WorkerMsg,
    p: &std::rc::Rc<std::cell::RefCell<LogProcessor>>,
    f: &std::rc::Rc<std::cell::RefCell<Option<String>>>,
    r: &web_sys::FileSystemDirectoryHandle,
    s: &web_sys::DedicatedWorkerGlobalScope,
) {
    let mut proc = p.borrow_mut();
    match msg {
        WorkerMsg::NewSession => {
            let (ff, rr, ss, pp) = (f.clone(), r.clone(), s.clone(), p.clone());
            spawn_local(async move {
                if let Ok(lock) = new_session(&rr, true, &mut ff.borrow_mut()).await {
                    let mut p_mut = pp.borrow_mut();
                    let _ = p_mut.set_sync_handle(lock);
                    let _ = p_mut.clear();
                    let _ = ss.post_message(
                        &serde_json::to_string(&WorkerMsg::TotalLines(0))
                            .unwrap()
                            .into(),
                    );
                }
            });
        }
        WorkerMsg::AppendChunk { chunk, is_hex } => {
            let _ = proc.append_chunk(&chunk, is_hex);
        }
        WorkerMsg::RequestWindow { start_line, count } => {
            if let Ok(val) = proc.request_window(start_line, count) {
                if let Ok(lines) = serde_wasm_bindgen::from_value::<Vec<String>>(val) {
                    let _ = s.post_message(
                        &serde_json::to_string(&WorkerMsg::LogWindow { start_line, lines })
                            .unwrap()
                            .into(),
                    );
                }
            }
        }
        WorkerMsg::Clear => {
            let _ = proc.clear();
            let _ = s.post_message(
                &serde_json::to_string(&WorkerMsg::TotalLines(0))
                    .unwrap()
                    .into(),
            );
        }
        WorkerMsg::SetLineEnding(mode) => proc.set_line_ending(&mode),
        WorkerMsg::SearchLogs {
            query,
            match_case,
            use_regex,
            invert,
        } => {
            if let Ok(c) = proc.search_logs(query, match_case, use_regex, invert) {
                let _ = s.post_message(
                    &serde_json::to_string(&WorkerMsg::TotalLines(c as usize))
                        .unwrap()
                        .into(),
                );
            }
        }
        WorkerMsg::ExportLogs { include_timestamp } => {
            if let Ok(stream) = proc.export_logs(include_timestamp) {
                let resp = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&resp, &"type".into(), &"EXPORT_STREAM".into());
                let _ = js_sys::Reflect::set(&resp, &"stream".into(), &stream);
                let _ = s.post_message_with_transfer(&resp, &js_sys::Array::of1(&stream));
            }
        }
        _ => {}
    }
}

#[wasm_bindgen(inline_js = r#"
export function get_current_script_url() {
    const scripts = Array.from(document.querySelectorAll('script[type="module"]'));
    const appScript = scripts.find(s => (s.src.includes('serial_monitor') || s.src.includes('web_serial_monitor')) && !s.src.includes('snippets'));
    return appScript ? appScript.src : "./serial_monitor.js";
}
"#)]
extern "C" {
    fn get_current_script_url() -> String;
}

pub fn get_app_script_path() -> String {
    get_current_script_url()
}
