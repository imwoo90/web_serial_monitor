use crate::state::LineEnding;
use chrono::Timelike;
use regex::Regex;
use std::borrow::Cow;
use std::fmt::Write;
use wasm_bindgen::prelude::*;
use wasm_streams::ReadableStream;
use web_sys::{FileSystemSyncAccessHandle, TextDecoder, TextEncoder};

/// Represents a byte range within the log file
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineRange {
    pub start: u64,
    pub end: u64,
}

/// Active filtering configuration
#[derive(Clone)]
pub struct ActiveFilter {
    pub query: String,
    pub lower_query: String,
    pub match_case: bool,
    pub regex: Option<Regex>,
    pub invert: bool,
}

impl ActiveFilter {
    pub fn matches(&self, text: &str) -> bool {
        let matched = if let Some(re) = &self.regex {
            re.is_match(text)
        } else if self.match_case {
            text.contains(&self.query)
        } else {
            text.to_lowercase().contains(&self.lower_query)
        };
        if self.invert {
            !matched
        } else {
            matched
        }
    }
}

#[wasm_bindgen]
pub struct LogProcessor {
    sync_handle: Option<FileSystemSyncAccessHandle>,
    // Merged CoreProcessor fields
    line_offsets: Vec<u64>,
    line_count: usize,
    filtered_lines: Vec<LineRange>,
    is_filtering: bool,
    active_filter: Option<ActiveFilter>,
    leftover_chunk: String,
    // Utils
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
            line_offsets: vec![0],
            line_count: 0,
            filtered_lines: Vec::new(),
            is_filtering: false,
            active_filter: None,
            leftover_chunk: String::new(),
            encoder: TextEncoder::new()?,
            decoder: TextDecoder::new()?,
            line_ending_mode: LineEnding::NL,
        })
    }

    pub fn get_line_count(&self) -> u32 {
        (if self.is_filtering {
            self.filtered_lines.len()
        } else {
            self.line_count
        }) as u32
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
        let file_size = handle.get_size()? as u64;
        self.sync_handle = Some(handle);

        if file_size > 0 {
            self.line_offsets = vec![0];
            self.line_count = 0;
            let mut offset = 0;
            let mut buf = vec![0u8; 64 * 1024];

            while offset < file_size {
                let read_len = (file_size - offset).min(buf.len() as u64) as usize;
                let slice = &mut buf[0..read_len];
                let opts = web_sys::FileSystemReadWriteOptions::new();
                opts.set_at(offset as f64);
                self.sync_handle
                    .as_ref()
                    .unwrap()
                    .read_with_u8_array_and_options(slice, &opts)?;

                for (i, &byte) in slice.iter().enumerate() {
                    if byte == 10 {
                        self.line_offsets.push(offset + i as u64 + 1);
                        self.line_count += 1;
                    }
                }
                offset += read_len as u64;
            }
        }
        Ok(())
    }

    pub fn append_chunk(&mut self, chunk: &[u8], is_hex: bool) -> Result<u32, JsValue> {
        let decoded_text = if is_hex {
            let mut hex = chunk
                .iter()
                .fold(String::new(), |acc, b| acc + &format!("{:02X} ", b));
            hex.push('\n');
            hex
        } else {
            let opts = web_sys::TextDecodeOptions::new();
            opts.set_stream(true);
            self.decoder
                .decode_with_u8_array_and_options(chunk, &opts)?
        };

        let (batch_str, new_offsets, new_filtered) = self.prepare_batch(&decoded_text);
        if batch_str.is_empty() {
            return Ok(self.get_line_count());
        }

        let start_pos = {
            let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;
            let start_pos = handle.get_size()? as u64;
            let opts = web_sys::FileSystemReadWriteOptions::new();
            opts.set_at(start_pos as f64);
            handle.write_with_u8_array_and_options(
                self.encoder.encode_with_input(&batch_str).as_ref(),
                &opts,
            )?;
            start_pos
        };

        for off in new_offsets {
            self.line_offsets.push(start_pos + off);
            self.line_count += 1;
        }
        for mut range in new_filtered {
            range.start += start_pos;
            range.end += start_pos;
            self.filtered_lines.push(range);
        }
        Ok(self.get_line_count())
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
        let start_pos = handle.get_size()? as u64;

        let opts = web_sys::FileSystemReadWriteOptions::new();
        opts.set_at(start_pos as f64);
        handle.write_with_u8_array_and_options(
            self.encoder.encode_with_input(&final_line).as_ref(),
            &opts,
        )?;

        self.line_offsets.push(start_pos + bytes_len);
        self.line_count += 1;

        if self.is_filtering {
            if let Some(filter) = &self.active_filter {
                if filter.matches(&final_line) {
                    self.filtered_lines.push(LineRange {
                        start: start_pos,
                        end: start_pos + bytes_len,
                    });
                }
            }
        }
        Ok(self.get_line_count())
    }

    fn prepare_batch(&mut self, chunk_text: &str) -> (String, Vec<u64>, Vec<LineRange>) {
        if !self.leftover_chunk.is_empty() && self.leftover_chunk.len() > 4096 {
            self.leftover_chunk.push('\n');
        }
        let full_text = if self.leftover_chunk.is_empty() {
            Cow::Borrowed(chunk_text)
        } else {
            Cow::Owned(format!("{}{}", self.leftover_chunk, chunk_text))
        };

        let mut lines_iter = match self.line_ending_mode {
            LineEnding::None | LineEnding::NL => full_text.split("\n"),
            LineEnding::CR => full_text.split("\r"),
            LineEnding::NLCR => full_text.split("\r\n"),
        }
        .peekable();

        let mut batch_buffer = String::with_capacity(chunk_text.len() * 2);
        let (mut new_offsets, mut new_filtered) = (Vec::new(), Vec::new());
        let mut relative_pos = 0u64;

        while let Some(line) = lines_iter.next() {
            if lines_iter.peek().is_none() {
                self.leftover_chunk = line.to_string();
                break;
            }

            let mut clean_line = line;
            if self.line_ending_mode == LineEnding::NL && clean_line.ends_with('\r') {
                clean_line = &clean_line[..clean_line.len() - 1];
            }
            if self.line_ending_mode == LineEnding::CR && clean_line.starts_with('\n') {
                clean_line = &clean_line[1..];
            }

            let start_len = batch_buffer.len();
            let now = chrono::Utc::now();
            let _ = write!(
                batch_buffer,
                "[{:02}:{:02}:{:02}.{:03}] {}\n",
                now.hour(),
                now.minute(),
                now.second(),
                now.timestamp_subsec_millis(),
                clean_line
            );

            let added_len = (batch_buffer.len() - start_len) as u64;
            if self.is_filtering {
                if let Some(f) = &self.active_filter {
                    if f.matches(&batch_buffer[start_len..]) {
                        new_filtered.push(LineRange {
                            start: relative_pos,
                            end: relative_pos + added_len,
                        });
                    }
                }
            }
            relative_pos += added_len;
            new_offsets.push(relative_pos);
        }
        (batch_buffer, new_offsets, new_filtered)
    }

    pub fn request_window(&self, start_line: usize, count: usize) -> Result<JsValue, JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;
        let total = self.get_line_count() as usize;
        let (start, end) = (start_line.min(total), (start_line + count).min(total));
        let mut lines = Vec::with_capacity(end - start);

        for i in start..end {
            let (s_off, e_off) = if self.is_filtering {
                let m = self.filtered_lines[i];
                (m.start, m.end)
            } else {
                (self.line_offsets[i], self.line_offsets[i + 1])
            };
            let mut buf = vec![0u8; (e_off - s_off) as usize];
            let opts = web_sys::FileSystemReadWriteOptions::new();
            opts.set_at(s_off as f64);
            handle.read_with_u8_array_and_options(&mut buf, &opts)?;
            let text = web_sys::TextDecoder::new()?.decode_with_u8_array(&buf)?;
            lines.push(text.trim_end_matches('\n').to_string());
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
            self.is_filtering = false;
            self.active_filter = None;
            self.filtered_lines.clear();
            return Ok(self.line_count as u32);
        }
        let regex = if use_regex {
            Some(
                regex::RegexBuilder::new(&query)
                    .case_insensitive(!match_case)
                    .build()
                    .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };
        self.active_filter = Some(ActiveFilter {
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
        self.is_filtering = true;
        self.filtered_lines.clear();

        let handle = self.sync_handle.as_ref().unwrap();
        let mut buf = vec![0u8; 512 * 1024];
        let mut idx = 0;
        while idx < self.line_count {
            let batch_end = (idx + 5000).min(self.line_count);
            let (s_off, e_off) = (self.line_offsets[idx], self.line_offsets[batch_end]);
            let size = (e_off - s_off) as usize;
            if buf.len() < size {
                buf.resize(size, 0);
            }
            let slice = &mut buf[0..size];
            let opts = web_sys::FileSystemReadWriteOptions::new();
            opts.set_at(s_off as f64);
            handle.read_with_u8_array_and_options(slice, &opts)?;

            let text = web_sys::TextDecoder::new()?.decode_with_u8_array(slice)?;
            let filter = self.active_filter.as_ref().unwrap();
            for (j, line) in text.trim_end_matches('\n').split('\n').enumerate() {
                if filter.matches(line) {
                    self.filtered_lines.push(LineRange {
                        start: self.line_offsets[idx + j],
                        end: self.line_offsets[idx + j + 1],
                    });
                }
            }
            idx = batch_end;
        }
        Ok(self.filtered_lines.len() as u32)
    }

    pub fn clear(&mut self) -> Result<(), JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?;
        handle.truncate_with_f64(0.0)?;
        handle.flush()?;
        self.line_offsets = vec![0];
        self.line_count = 0;
        self.filtered_lines.clear();
        Ok(())
    }

    pub fn export_logs(&self, include_timestamps: bool) -> Result<js_sys::Object, JsValue> {
        let handle = self.sync_handle.as_ref().ok_or("No sync handle")?.clone();
        let file_size = handle.get_size()? as u64;
        let (decoder, encoder, line_ending) = (
            self.decoder.clone(),
            self.encoder.clone(),
            self.line_ending_mode,
        );

        let stream = futures_util::stream::unfold(0u64, move |offset| {
            let (handle, decoder, encoder) = (handle.clone(), decoder.clone(), encoder.clone());
            async move {
                if offset >= file_size {
                    return None;
                }
                let read_len = (file_size - offset).min(64 * 1024) as usize;
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
                    Some((
                        Ok(JsValue::from(js_sys::Uint8Array::from(&buf[..]))),
                        offset + read_len as u64,
                    ))
                } else {
                    let text = decoder.decode_with_u8_array(&buf).unwrap_or_default();
                    let sep = match line_ending {
                        LineEnding::CR => "\r",
                        LineEnding::NLCR => "\r\n",
                        _ => "\n",
                    };
                    let result = text
                        .split(sep)
                        .map(|l| if l.len() > 15 { &l[15..] } else { l })
                        .collect::<Vec<_>>()
                        .join(sep);
                    Some((
                        Ok(JsValue::from(encoder.encode_with_input(&result))),
                        offset + read_len as u64,
                    ))
                }
            }
        });
        Ok(ReadableStream::from_stream(stream).into_raw().into())
    }
}
