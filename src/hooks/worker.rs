use crate::state::AppState;
use crate::worker::types::WorkerMsg;
use dioxus::prelude::*;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

pub fn use_log_worker(mut state: AppState) {
    use_effect(move || {
        if state.conn.log_worker.read().is_none() {
            let script_path = crate::worker::get_app_script_path();
            let options = web_sys::WorkerOptions::new();
            options.set_type(web_sys::WorkerType::Module);

            if let Ok(worker) = web_sys::Worker::new_with_options(&script_path, &options) {
                let mut tl = state.log.total_lines;
                let mut vl = state.log.visible_logs;

                let callback = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
                    let data = event.data();

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

                    if let Some(msg_str) = data.as_string() {
                        if let Ok(msg) = serde_json::from_str::<WorkerMsg>(&msg_str) {
                            match msg {
                                WorkerMsg::TotalLines(count) => {
                                    tl.set(count);
                                    if count == 0 {
                                        vl.set(Vec::new());
                                    }
                                }
                                WorkerMsg::LogWindow { lines, .. } => vl.set(lines),
                                WorkerMsg::Error(msg) => {
                                    if let Some(win) = web_sys::window() {
                                        let _ = win
                                            .alert_with_message(&format!("Worker Error: {}", msg));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                worker.set_onmessage(Some(callback.as_ref().unchecked_ref()));
                callback.forget();
                state.conn.log_worker.set(Some(worker));
            }
        }
    });

    // RX Line Ending Sync
    use_effect(move || {
        let ending = (state.serial.rx_line_ending)();
        if let Some(w) = state.conn.log_worker.read().as_ref() {
            let mode_str = match ending {
                crate::state::LineEnding::None => "None",
                crate::state::LineEnding::NL => "NL",
                crate::state::LineEnding::CR => "CR",
                crate::state::LineEnding::NLCR => "NLCR",
            };
            crate::utils::send_worker_msg(w, WorkerMsg::SetLineEnding(mode_str.to_string()));
        }
    });
}
