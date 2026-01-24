use crate::state::LineEnding;
use crate::worker::core::{ActiveFilter, CoreProcessor, LineRange};
use chrono::Timelike;
use wasm_bindgen::prelude::*;

use wasm_streams::ReadableStream;
use web_sys::{FileSystemSyncAccessHandle, TextDecoder, TextEncoder};

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

        let (batch_str, new_offsets, new_filtered) = self
            .core
            .prepare_batch(&decoded_text, self.line_ending_mode);

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
        let mut buf = Vec::with_capacity(1024 * 512); // Start with 512KB

        while idx < self.core.line_count {
            let batch_end = (idx + batch_size_lines).min(self.core.line_count);
            let start_off = self.core.line_offsets[idx];
            let end_off = self.core.line_offsets[batch_end];
            let size = (end_off - start_off) as usize;

            if buf.len() < size {
                buf.resize(size, 0);
            }
            let slice = &mut buf[0..size];

            let opts = web_sys::FileSystemReadWriteOptions::new();
            opts.set_at(start_off as f64);
            handle.read_with_u8_array_and_options(slice, &opts)?;

            let text = web_sys::TextDecoder::new()?.decode_with_u8_array(slice)?;
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
