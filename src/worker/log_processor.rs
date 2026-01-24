use crate::components::console::types::WorkerMsg;
use crate::state::LineEnding;
use chrono::Timelike;
use regex::Regex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_streams::ReadableStream;
use web_sys::{FileSystemSyncAccessHandle, TextDecoder, TextEncoder};

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
/// Represents a byte range within the log file
#[derive(Clone, Copy, Debug, PartialEq)]
struct LineRange {
    start: u64,
    end: u64,
}

/// Active filtering configuration
#[derive(Clone)]
struct ActiveFilter {
    query: String,
    lower_query: String,
    match_case: bool,
    regex: Option<Regex>,
    invert: bool,
}

impl ActiveFilter {
    fn matches(&self, text: &str) -> bool {
        let mut matched = if let Some(re) = &self.regex {
            re.is_match(text)
        } else if self.match_case {
            text.contains(&self.query)
        } else {
            text.to_lowercase().contains(&self.lower_query)
        };

        if self.invert {
            matched = !matched;
        }
        matched
    }
}

/// Core processing logic separated from WASM/IO for testability
struct CoreProcessor {
    line_offsets: Vec<u64>,
    line_count: usize,
    filtered_lines: Vec<LineRange>,
    is_filtering: bool,
    active_filter: Option<ActiveFilter>,
    leftover_chunk: String,
}

impl CoreProcessor {
    fn new() -> Self {
        Self {
            line_offsets: vec![0],
            line_count: 0,
            filtered_lines: Vec::new(),
            is_filtering: false,
            active_filter: None,
            leftover_chunk: String::new(),
        }
    }

    /// Processes decoded text and prepares formatted lines with timestamps.
    /// Returns (Formatted Buffer String, Tuple of (LineOffsetsToAdd, FilteredLinesToAdd))
    fn prepare_batch(
        &mut self,
        chunk_text: String,
        line_ending_mode: LineEnding,
    ) -> (String, Vec<u64>, Vec<LineRange>) {
        if self.leftover_chunk.len() > 4 * 1024 {
            // Safety: If buffer grows too large without newline, force a flush (e.g. 4KB)
            self.leftover_chunk.push('\n');
        }

        let full_text = format!("{}{}", self.leftover_chunk, chunk_text);

        let mut lines: Vec<&str> = match line_ending_mode {
            LineEnding::None => full_text.split('\n').collect(),
            LineEnding::CR => full_text.split('\r').collect(),
            LineEnding::NLCR => full_text.split("\r\n").collect(),
            LineEnding::NL => full_text.split('\n').collect(),
        };

        if let Some(last) = lines.pop() {
            self.leftover_chunk = last.to_string();
        } else {
            self.leftover_chunk.clear();
        }

        if lines.is_empty() {
            return (String::new(), Vec::new(), Vec::new());
        }

        let now = chrono::Utc::now();
        let time_prefix = format!(
            "[{:02}:{:02}:{:02}.{:03}] ",
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_subsec_millis()
        );

        let mut batch_buffer = String::new();
        let mut new_offsets = Vec::new();
        let mut new_filtered = Vec::new();

        // This pos is not accurate here because it depends on file size,
        // will be adjusted by LogProcessor.
        let mut relative_pos = 0u64;

        for line in lines {
            let mut clean_line = line;
            if line_ending_mode == LineEnding::NL && clean_line.ends_with('\r') {
                clean_line = &clean_line[..clean_line.len() - 1];
            }
            if line_ending_mode == LineEnding::CR && clean_line.starts_with('\n') {
                clean_line = &clean_line[1..];
            }

            let final_line = format!("{}{}\n", time_prefix, clean_line);
            let bytes_len = final_line.len() as u64;

            if self.is_filtering {
                if let Some(filter) = &self.active_filter {
                    if filter.matches(&final_line) {
                        new_filtered.push(LineRange {
                            start: relative_pos,
                            end: relative_pos + bytes_len,
                        });
                    }
                }
            }

            batch_buffer.push_str(&final_line);
            relative_pos += bytes_len;
            new_offsets.push(relative_pos);
        }

        (batch_buffer, new_offsets, new_filtered)
    }

    fn get_total_lines(&self) -> usize {
        if self.is_filtering {
            self.filtered_lines.len()
        } else {
            self.line_count
        }
    }
}

#[wasm_bindgen]
pub struct LogProcessor {
    sync_handle: Option<FileSystemSyncAccessHandle>,
    core: CoreProcessor,
    encoder: TextEncoder,
    decoder: TextDecoder,
    line_ending_mode: LineEnding,
}

#[wasm_bindgen]
impl LogProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<LogProcessor, JsValue> {
        Ok(LogProcessor {
            sync_handle: None,
            core: CoreProcessor::new(),
            encoder: TextEncoder::new()?,
            decoder: TextDecoder::new()?,
            line_ending_mode: LineEnding::NL,
        })
    }

    pub fn get_line_count(&self) -> u32 {
        self.core.get_total_lines() as u32
    }

    pub fn set_line_ending(&mut self, mode: &str) {
        self.line_ending_mode = match mode {
            "None" => LineEnding::None,
            "NL" => LineEnding::NL,
            "CR" => LineEnding::CR,
            "NLCR" => LineEnding::NLCR,
            _ => LineEnding::NL,
        };
    }

    pub fn set_sync_handle(&mut self, handle: FileSystemSyncAccessHandle) -> Result<(), JsValue> {
        self.sync_handle = Some(handle);

        let handle = self.sync_handle.as_ref().unwrap();
        let file_size = handle.get_size()? as u64;

        if file_size > 0 {
            // Rebuild indices from existing file
            self.core.line_offsets.clear();
            self.core.line_offsets.push(0);
            self.core.line_count = 0;

            let chunk_size = 64 * 1024;
            let mut offset = 0;
            let mut buf = vec![0u8; chunk_size];

            while offset < file_size {
                let read_len = (file_size - offset).min(chunk_size as u64) as usize;
                let slice = &mut buf[0..read_len];
                let opts = web_sys::FileSystemReadWriteOptions::new();
                opts.set_at(offset as f64);

                handle.read_with_u8_array_and_options(slice, &opts)?;

                for (i, byte) in slice.iter().enumerate() {
                    if *byte == 10 {
                        // \n
                        let global_pos = offset + i as u64 + 1;
                        self.core.line_offsets.push(global_pos);
                        self.core.line_count += 1;
                    }
                }
                offset += read_len as u64;
            }
        }
        Ok(())
    }

    pub fn append_chunk(&mut self, chunk: &[u8], is_hex: bool) -> Result<u32, JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;

        let decoded_text = if is_hex {
            let hex = chunk
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{}\n", hex)
        } else {
            let opts = web_sys::TextDecodeOptions::new();
            opts.set_stream(true);
            self.decoder
                .decode_with_u8_array_and_options(chunk, &opts)?
        };

        let (batch_str, new_offsets, new_filtered) =
            self.core.prepare_batch(decoded_text, self.line_ending_mode);

        if batch_str.is_empty() {
            return Ok(self.core.get_total_lines() as u32);
        }

        let write_buffer = self.encoder.encode_with_input(&batch_str);
        let start_pos = handle.get_size()? as u64;

        let opts = web_sys::FileSystemReadWriteOptions::new();
        opts.set_at(start_pos as f64);
        handle.write_with_u8_array_and_options(write_buffer.as_ref(), &opts)?;

        // Adjust and store offsets
        for off in new_offsets {
            self.core.line_offsets.push(start_pos + off);
            self.core.line_count += 1;
        }

        // Adjust and store filtered lines
        for mut range in new_filtered {
            range.start += start_pos;
            range.end += start_pos;
            self.core.filtered_lines.push(range);
        }

        Ok(self.core.get_total_lines() as u32)
    }

    pub fn append_log(&mut self, text: String) -> Result<u32, JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;

        let now = chrono::Utc::now();
        let time_prefix = format!(
            "[{:02}:{:02}:{:02}.{:03}] ",
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_subsec_millis()
        );

        let final_line = format!("[TX] {}{}\n", time_prefix, text);
        let bytes_len = final_line.len() as u64;

        let write_buffer = self.encoder.encode_with_input(&final_line);
        let start_pos = handle.get_size()? as u64;

        let opts = web_sys::FileSystemReadWriteOptions::new();
        opts.set_at(start_pos as f64);
        handle.write_with_u8_array_and_options(write_buffer.as_ref(), &opts)?;

        self.core.line_offsets.push(start_pos + bytes_len);
        self.core.line_count += 1;

        if self.core.is_filtering {
            if let Some(filter) = &self.core.active_filter {
                if filter.matches(&final_line) {
                    self.core.filtered_lines.push(LineRange {
                        start: start_pos,
                        end: start_pos + bytes_len,
                    });
                }
            }
        }

        Ok(self.core.get_total_lines() as u32)
    }

    pub fn request_window(&self, start_line: usize, count: usize) -> Result<JsValue, JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;
        let total = self.core.get_total_lines();
        let start = start_line.min(total);
        let end = (start + count).min(total);
        let effective_count = end - start;

        if effective_count == 0 {
            return Ok(serde_wasm_bindgen::to_value(&Vec::<String>::new())?);
        }

        let mut lines = Vec::with_capacity(effective_count);
        if self.core.is_filtering {
            for i in start..end {
                let meta = &self.core.filtered_lines[i];
                let size = (meta.end - meta.start) as usize;
                let mut buf = vec![0u8; size];
                let opts = web_sys::FileSystemReadWriteOptions::new();
                opts.set_at(meta.start as f64);
                handle.read_with_u8_array_and_options(&mut buf, &opts)?;
                let text = web_sys::TextDecoder::new()?.decode_with_u8_array(&buf)?;
                lines.push(if text.ends_with('\n') {
                    text[..text.len() - 1].to_string()
                } else {
                    text
                });
            }
        } else {
            let start_offset = self.core.line_offsets[start];
            let end_offset = self.core.line_offsets[end];
            let size = (end_offset - start_offset) as usize;
            let mut read_buffer = vec![0u8; size];
            let opts = web_sys::FileSystemReadWriteOptions::new();
            opts.set_at(start_offset as f64);
            handle.read_with_u8_array_and_options(&mut read_buffer, &opts)?;
            let text = web_sys::TextDecoder::new()?.decode_with_u8_array(&read_buffer)?;
            let split = if text.ends_with('\n') {
                &text[..text.len() - 1]
            } else {
                &text
            };
            for l in split.split('\n') {
                lines.push(l.to_string());
            }
        }

        Ok(serde_wasm_bindgen::to_value(&lines)?)
    }

    pub fn search_logs(
        &mut self,
        query: String,
        match_case: bool,
        use_regex: bool,
        invert: bool,
    ) -> Result<u32, JsValue> {
        if query.trim().is_empty() {
            self.core.is_filtering = false;
            self.core.active_filter = None;
            self.core.filtered_lines.clear();
            return Ok(self.core.line_count as u32);
        }

        let regex = if use_regex {
            Some(
                regex::RegexBuilder::new(&query)
                    .case_insensitive(!match_case)
                    .build()
                    .map_err(|e| format!("Invalid regex: {}", e))?,
            )
        } else {
            None
        };

        self.core.active_filter = Some(ActiveFilter {
            lower_query: if match_case {
                query.clone()
            } else {
                query.to_lowercase()
            },
            query,
            match_case,
            regex,
            invert,
        });

        self.core.is_filtering = true;
        self.core.filtered_lines.clear();

        // Perform full scan from file
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;
        let batch_size_lines = 5000;
        let mut idx = 0;
        while idx < self.core.line_count {
            let batch_end = (idx + batch_size_lines).min(self.core.line_count);
            let start_off = self.core.line_offsets[idx];
            let end_off = self.core.line_offsets[batch_end];
            let size = (end_off - start_off) as usize;

            let mut buf = vec![0u8; size];
            let opts = web_sys::FileSystemReadWriteOptions::new();
            opts.set_at(start_off as f64);
            handle.read_with_u8_array_and_options(&mut buf, &opts)?;

            let text = web_sys::TextDecoder::new()?.decode_with_u8_array(&buf)?;
            let clean_text = if text.ends_with('\n') {
                &text[..text.len() - 1]
            } else {
                &text
            };

            let filter = self.core.active_filter.as_ref().unwrap();
            for (j, line) in clean_text.split('\n').enumerate() {
                if filter.matches(line) {
                    let global_line_idx = idx + j;
                    self.core.filtered_lines.push(LineRange {
                        start: self.core.line_offsets[global_line_idx],
                        end: self.core.line_offsets[global_line_idx + 1],
                    });
                }
            }
            idx = batch_end;
        }

        Ok(self.core.filtered_lines.len() as u32)
    }

    pub fn clear(&mut self) -> Result<(), JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;
        handle.truncate_with_f64(0.0)?;
        handle.flush()?;
        self.core = CoreProcessor::new();
        Ok(())
    }

    pub fn export_logs(&self, include_timestamps: bool) -> Result<js_sys::Object, JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?.clone();
        let file_size = handle.get_size()? as u64;
        let decoder = self.decoder.clone();
        let encoder = self.encoder.clone();
        let line_ending = self.line_ending_mode;

        let stream = futures_util::stream::unfold(0u64, move |offset| {
            let handle = handle.clone();
            let decoder = decoder.clone();
            let encoder = encoder.clone();

            async move {
                if offset >= file_size {
                    return None;
                }

                let chunk_size = 64 * 1024;
                let read_len = (file_size - offset).min(chunk_size as u64) as usize;
                let mut buf = vec![0u8; read_len];

                let opts = web_sys::FileSystemReadWriteOptions::new();
                opts.set_at(offset as f64);

                if handle
                    .read_with_u8_array_and_options(&mut buf, &opts)
                    .is_err()
                {
                    return None;
                }

                if include_timestamps {
                    let js_buf = js_sys::Uint8Array::from(&buf[..]);
                    Some((Ok(JsValue::from(js_buf)), offset + read_len as u64))
                } else {
                    let text = decoder.decode_with_u8_array(&buf).unwrap_or_default();
                    let separator = match line_ending {
                        LineEnding::CR => "\r",
                        LineEnding::NLCR => "\r\n",
                        _ => "\n",
                    };

                    let mut result = String::with_capacity(text.len());
                    for line in text.split(separator) {
                        if line.len() > 15 {
                            result.push_str(&line[15..]);
                            result.push_str(separator);
                        } else if !line.is_empty() {
                            result.push_str(line);
                            result.push_str(separator);
                        }
                    }
                    let encoded = encoder.encode_with_input(&result);
                    Some((Ok(JsValue::from(encoded)), offset + read_len as u64))
                }
            }
        });

        let readable = ReadableStream::from_stream(stream);
        Ok(readable.into_raw().into())
    }
}
async fn get_opfs_root() -> Result<web_sys::FileSystemDirectoryHandle, JsValue> {
    let global = js_sys::global();
    let navigator = js_sys::Reflect::get(&global, &"navigator".into())?;
    let storage = js_sys::Reflect::get(&navigator, &"storage".into())?;
    let storage: web_sys::StorageManager = storage.unchecked_into();
    let root = wasm_bindgen_futures::JsFuture::from(storage.get_directory()).await?;
    Ok(root.into())
}

async fn get_lock(
    file_handle: web_sys::FileSystemFileHandle,
) -> Result<web_sys::FileSystemSyncAccessHandle, JsValue> {
    for _ in 0..20 {
        match wasm_bindgen_futures::JsFuture::from(file_handle.create_sync_access_handle()).await {
            Ok(h) => return Ok(h.into()),
            Err(e) => {
                let error_name = js_sys::Reflect::get(&e, &"name".into()).unwrap_or_default();
                if error_name == "NoModificationAllowedError" || error_name == "InvalidStateError" {
                    gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err("Failed to acquire OPFS lock after retries".into())
}

async fn get_files(
    root: &web_sys::FileSystemDirectoryHandle,
) -> Result<Vec<(String, web_sys::FileSystemFileHandle)>, JsValue> {
    let mut files = Vec::new();
    let entries_fn = js_sys::Reflect::get(root, &"entries".into())?;
    let iterator = js_sys::Function::from(entries_fn)
        .call0(root)?
        .unchecked_into::<js_sys::AsyncIterator>();

    loop {
        let result = wasm_bindgen_futures::JsFuture::from(iterator.next()?).await?;
        let done = js_sys::Reflect::get(&result, &"done".into())?
            .as_bool()
            .unwrap_or(true);
        if done {
            break;
        }
        let value = js_sys::Reflect::get(&result, &"value".into())?;
        let entry = value.unchecked_into::<js_sys::Array>();
        let name = entry.get(0).as_string().unwrap_or_default();
        if name.starts_with("logs_") && name.ends_with(".txt") {
            let handle = entry
                .get(1)
                .unchecked_into::<web_sys::FileSystemFileHandle>();
            files.push((name, handle));
        }
    }

    files.sort_by(|a, b| {
        let ts_a = a.0[5..a.0.len() - 4].parse::<u64>().unwrap_or(0);
        let ts_b = b.0[5..b.0.len() - 4].parse::<u64>().unwrap_or(0);
        ts_b.cmp(&ts_a)
    });

    Ok(files)
}

async fn new_session(
    root: &web_sys::FileSystemDirectoryHandle,
    cleanup_current: bool,
    current_filename: &mut Option<String>,
) -> Result<web_sys::FileSystemSyncAccessHandle, JsValue> {
    if cleanup_current {
        if let Some(name) = current_filename {
            let _ = wasm_bindgen_futures::JsFuture::from(root.remove_entry(name)).await;
        }
    }

    let filename = format!("logs_{}.txt", chrono::Utc::now().timestamp_millis());
    let opts = web_sys::FileSystemGetFileOptions::new();
    opts.set_create(true);
    let file_handle =
        wasm_bindgen_futures::JsFuture::from(root.get_file_handle_with_options(&filename, &opts))
            .await?;
    let file_handle: web_sys::FileSystemFileHandle = file_handle.into();

    let lock = get_lock(file_handle).await?;
    *current_filename = Some(filename);
    Ok(lock)
}

async fn setup_opfs_manual(
    processor: &mut LogProcessor,
    current_filename: &mut Option<String>,
) -> Result<(), JsValue> {
    let root = get_opfs_root().await?;
    let files = get_files(&root).await?;

    if let Some((name, handle)) = files.first().cloned() {
        match get_lock(handle).await {
            Ok(lock) => {
                processor.set_sync_handle(lock)?;
                *current_filename = Some(name);
            }
            Err(_) => {
                let lock = new_session(&root, false, current_filename).await?;
                processor.set_sync_handle(lock)?;
            }
        }
        // Cleanup old files
        for i in 1..files.len() {
            let _ = wasm_bindgen_futures::JsFuture::from(root.remove_entry(&files[i].0)).await;
        }
    } else {
        let lock = new_session(&root, false, current_filename).await?;
        processor.set_sync_handle(lock)?;
    }
    Ok(())
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
        let _ = setup_opfs_manual(&mut processor, &mut current_filename).await;

        let scope = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();
        let root = get_opfs_root().await.expect("Failed to get OPFS root");
        let mut last_count = 0;

        let processor_ptr = std::rc::Rc::new(std::cell::RefCell::new(processor));
        let filename_ptr = std::rc::Rc::new(std::cell::RefCell::new(current_filename));

        // Closures and clones for message handling
        let processor = processor_ptr.clone();
        let filename = filename_ptr.clone();
        let root = root.clone();
        let scope_for_msg = scope.clone();
        let scope_for_loop = scope.clone();

        let onmessage = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            if let Some(msg_str) = event.data().as_string() {
                if let Ok(msg) = serde_json::from_str::<WorkerMsg>(&msg_str) {
                    let mut p = processor.borrow_mut();
                    match msg {
                        WorkerMsg::NewSession => {
                            let filename_rc = filename.clone();
                            let root_rc = root.clone();
                            let scope_rc = scope_for_msg.clone();
                            let proc_for_spawn = processor.clone();
                            spawn_local(async move {
                                let lock_res = {
                                    let mut f = filename_rc.borrow_mut();
                                    new_session(&root_rc, true, &mut f).await
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
                                if let Ok(lines) =
                                    serde_wasm_bindgen::from_value::<Vec<String>>(val)
                                {
                                    let resp = WorkerMsg::LogWindow { start_line, lines };
                                    let _ = scope_for_msg.post_message(
                                        &serde_json::to_string(&resp).unwrap().into(),
                                    );
                                }
                            }
                        }
                        WorkerMsg::Clear => {
                            let _ = p.clear();
                            let _ = scope_for_msg.post_message(
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
                                let _ = scope_for_msg.post_message(
                                    &serde_json::to_string(&WorkerMsg::TotalLines(count as usize))
                                        .unwrap()
                                        .into(),
                                );
                            }
                        }
                        WorkerMsg::ExportLogs { include_timestamp } => {
                            if let Ok(stream) = p.export_logs(include_timestamp) {
                                let resp = js_sys::Object::new();
                                let _ = js_sys::Reflect::set(
                                    &resp,
                                    &"type".into(),
                                    &"EXPORT_STREAM".into(),
                                );
                                let _ = js_sys::Reflect::set(&resp, &"stream".into(), &stream);
                                let transfer = js_sys::Array::of1(&stream);
                                let _ = scope_for_msg.post_message_with_transfer(&resp, &transfer);
                            }
                        }
                        _ => {}
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

#[cfg(test)]
mod tests {
    // ... (rest of tests)
    use super::*;

    #[test]
    fn test_active_filter_matches() {
        let filter = ActiveFilter {
            query: "ERROR".into(),
            lower_query: "error".into(),
            match_case: true,
            regex: None,
            invert: false,
        };
        assert!(filter.matches("System ERROR occurred"));
        assert!(!filter.matches("system error occurred"));

        let filter_nocase = ActiveFilter {
            query: "ERROR".into(),
            lower_query: "error".into(),
            match_case: false,
            regex: None,
            invert: false,
        };
        assert!(filter_nocase.matches("system error occurred"));
    }

    #[test]
    fn test_prepare_batch_splitting() {
        let mut core = CoreProcessor::new();
        let (batch, offsets, _) =
            core.prepare_batch("Hello\nWorld\nIncompl".into(), LineEnding::NL);

        // Lines should be timestamped.
        // We can't predict exact timestamp but check format.
        let lines: Vec<&str> = batch.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("Hello"));
        assert!(lines[1].contains("World"));
        assert_eq!(core.leftover_chunk, "Incompl");
        assert_eq!(offsets.len(), 2);
    }

    #[test]
    fn test_filter_integration() {
        let mut core = CoreProcessor::new();
        core.is_filtering = true;
        core.active_filter = Some(ActiveFilter {
            query: "Critical".into(),
            lower_query: "critical".into(),
            match_case: true,
            regex: None,
            invert: false,
        });

        let (batch, _, filtered) =
            core.prepare_batch("Info: log\nCritical: error\n".into(), LineEnding::NL);

        assert_eq!(filtered.len(), 1);
        // The second line matches
        let lines: Vec<&str> = batch.lines().collect();
        assert!(lines[1].contains("Critical"));
    }
}
