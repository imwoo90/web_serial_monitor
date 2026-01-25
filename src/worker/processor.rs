use crate::state::LineEnding;
use crate::worker::error::LogError;
use crate::worker::formatter::{
    DefaultFormatter, HexFormatter, LogFormatter, LogFormatterStrategy,
};
use crate::worker::index::{ActiveFilterBuilder, ByteOffset, LineIndex, LineRange, LogIndex};
use crate::worker::storage::{OpfsBackend, StorageBackend};

use std::borrow::Cow;
use wasm_bindgen::prelude::*;
use wasm_streams::ReadableStream;
use web_sys::{FileSystemReadWriteOptions, FileSystemSyncAccessHandle, TextDecoder, TextEncoder};

const READ_BUFFER_SIZE: usize = 64 * 1024;
const SEARCH_BATCH_SIZE: usize = 5000;
const EXPORT_CHUNK_SIZE: u64 = 64 * 1024;
pub const MAX_LINE_BYTES: usize = 256;

struct LogStorage {
    backend: OpfsBackend,
    encoder: TextEncoder,
    decoder: TextDecoder,
}

impl LogStorage {
    fn new() -> Result<Self, LogError> {
        Ok(Self {
            backend: OpfsBackend { handle: None },
            encoder: TextEncoder::new().map_err(LogError::from)?,
            decoder: TextDecoder::new().map_err(LogError::from)?,
        })
    }
}

#[wasm_bindgen]
pub struct LogProcessor {
    storage: LogStorage,
    index: LogIndex,
    formatter: LogFormatter,
    leftover_chunk: String,
}

#[wasm_bindgen]
impl LogProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<LogProcessor, JsValue> {
        LogProcessor::new_internal().map_err(JsValue::from)
    }

    fn new_internal() -> Result<Self, LogError> {
        Ok(LogProcessor {
            storage: LogStorage::new()?,
            index: LogIndex::new(),
            formatter: LogFormatter::new(LineEnding::NL),
            leftover_chunk: String::new(),
        })
    }

    // --- Public API ---
    pub fn get_line_count(&self) -> u32 {
        self.index.get_total_count() as u32
    }

    pub fn set_line_ending(&mut self, mode: &str) {
        self.formatter.line_ending_mode = match mode {
            "None" => LineEnding::None,
            "NL" => LineEnding::NL,
            "CR" => LineEnding::CR,
            "NLCR" => LineEnding::NLCR,
            _ => LineEnding::NL,
        };
    }

    pub fn set_sync_handle(&mut self, handle: FileSystemSyncAccessHandle) -> Result<(), JsValue> {
        self.set_sync_handle_internal(handle).map_err(JsValue::from)
    }

    fn set_sync_handle_internal(
        &mut self,
        handle: FileSystemSyncAccessHandle,
    ) -> Result<(), LogError> {
        self.storage.backend.handle = Some(handle);
        let size = self.storage.backend.get_file_size()?;
        if size.0 > 0 {
            self.index.reset_base();
            let (mut off, mut buf) = (ByteOffset(0), vec![0u8; READ_BUFFER_SIZE]);
            while off.0 < size.0 {
                let len = (size.0 - off.0).min(buf.len() as u64) as usize;
                self.storage.backend.read_at(off, &mut buf[..len])?;
                for (i, &b) in buf[..len].iter().enumerate() {
                    if b == 10 {
                        self.index.push_line(off + (i as u64 + 1));
                    }
                }
                off = off + (len as u64);
            }
        }
        Ok(())
    }

    pub fn append_chunk(&mut self, chunk: &[u8], is_hex: bool) -> Result<u32, JsValue> {
        let formatter: Box<dyn LogFormatterStrategy> = if is_hex {
            Box::new(HexFormatter {
                line_ending: self.formatter.line_ending_mode,
                max_bytes: MAX_LINE_BYTES,
            })
        } else {
            Box::new(DefaultFormatter {
                line_ending: self.formatter.line_ending_mode,
                max_bytes: MAX_LINE_BYTES,
            })
        };

        let text = if is_hex {
            formatter.format_chunk(chunk)
        } else {
            self.decode_with_streaming_internal(chunk)?
        };

        let (batch, offsets, filtered) = self.prepare_batch_with_formatter(&text, &*formatter);
        if !batch.is_empty() {
            self.write_and_update_internal(&batch, offsets, filtered)
                .map_err(JsValue::from)?;
        }
        Ok(self.get_line_count())
    }

    pub fn append_log(&mut self, text: String) -> Result<u32, JsValue> {
        let log = format!("[TX] {} {}\n", self.formatter.get_timestamp(), text);
        let len = ByteOffset(log.len() as u64);
        let filtered = if self.index.is_filtering
            && self
                .index
                .active_filter
                .as_ref()
                .map_or(false, |f| f.matches(&log))
        {
            vec![LineRange {
                start: ByteOffset(0),
                end: len,
            }]
        } else {
            vec![]
        };
        self.write_and_update_internal(&log, vec![len], filtered)
            .map_err(JsValue::from)?;
        Ok(self.get_line_count())
    }

    pub fn request_window(&self, start: usize, count: usize) -> Result<JsValue, JsValue> {
        self.request_window_internal(start, count)
            .map_err(JsValue::from)
    }

    fn request_window_internal(&self, start: usize, count: usize) -> Result<JsValue, LogError> {
        let total = self.get_line_count() as usize;
        let (s, e) = (start.min(total), (start + count).min(total));
        let mut lines = Vec::with_capacity(e - s);
        for i in s..e {
            if let Some(range) = self.index.get_line_range(LineIndex(i)) {
                let mut buf = vec![0u8; (range.end.0 - range.start.0) as usize];
                self.storage.backend.read_at(range.start, &mut buf)?;
                lines.push(
                    self.storage
                        .decoder
                        .decode_with_u8_array(&buf)
                        .map_err(LogError::from)?
                        .trim_end_matches('\n')
                        .to_string(),
                );
            }
        }
        Ok(serde_wasm_bindgen::to_value(&lines).map_err(|e| LogError::Encoding(e.to_string()))?)
    }

    pub fn search_logs(
        &mut self,
        query: String,
        case: bool,
        regex: bool,
        invert: bool,
    ) -> Result<u32, JsValue> {
        self.search_logs_internal(query, case, regex, invert)
            .map_err(JsValue::from)
    }

    fn search_logs_internal(
        &mut self,
        query: String,
        case: bool,
        regex: bool,
        invert: bool,
    ) -> Result<u32, LogError> {
        if query.trim().is_empty() {
            return self.reset_filter_internal();
        }

        self.index.active_filter = Some(
            ActiveFilterBuilder::new(query)
                .case_sensitive(case)
                .regex(regex)
                .invert(invert)
                .build()
                .map_err(LogError::Regex)?,
        );
        self.index.is_filtering = true;
        self.index.filtered_lines.clear();

        let total_lines = self.index.line_count;
        let mut buf = vec![0u8; 512 * 1024];
        let mut idx = 0;

        while idx < total_lines {
            let batch_end = (idx + SEARCH_BATCH_SIZE).min(total_lines);
            let (s_off, e_off) = {
                let off = &self.index.line_offsets;
                (off[idx], off[batch_end])
            };
            let size = (e_off.0 - s_off.0) as usize;
            if buf.len() < size {
                buf.resize(size, 0);
            }
            self.storage.backend.read_at(s_off, &mut buf[..size])?;

            let text = self
                .storage
                .decoder
                .decode_with_u8_array(&buf[..size])
                .map_err(LogError::Js)?;
            let filter = self.index.active_filter.as_ref().unwrap().clone();

            for (j, line) in text.trim_end_matches('\n').split('\n').enumerate() {
                if filter.matches(line) {
                    let off_ptr = &self.index.line_offsets;
                    let range = LineRange {
                        start: off_ptr[idx + j],
                        end: off_ptr[idx + j + 1],
                    };
                    self.index.push_filtered(range);
                }
            }
            idx = batch_end;
        }
        Ok(self.index.filtered_lines.len() as u32)
    }

    pub fn clear(&mut self) -> Result<(), JsValue> {
        self.clear_internal().map_err(JsValue::from)
    }

    fn clear_internal(&mut self) -> Result<(), LogError> {
        self.storage.backend.truncate(0)?;
        self.storage.backend.flush()?;
        self.index.reset_base();
        Ok(())
    }

    pub fn export_logs(&self, ts: bool) -> Result<js_sys::Object, JsValue> {
        self.export_logs_internal(ts).map_err(JsValue::from)
    }

    fn export_logs_internal(&self, ts: bool) -> Result<js_sys::Object, LogError> {
        let size = self.storage.backend.get_file_size()?;
        let (dec, enc, mode, backend) = (
            self.storage.decoder.clone(),
            self.storage.encoder.clone(),
            self.formatter.line_ending_mode,
            self.storage.backend.handle.as_ref().cloned().unwrap(),
        );

        let stream = futures_util::stream::unfold(ByteOffset(0), move |off| {
            let (h, d, e) = (backend.clone(), dec.clone(), enc.clone());
            async move {
                if off.0 >= size.0 {
                    return None;
                }
                let len = (size.0 - off.0).min(EXPORT_CHUNK_SIZE) as usize;
                let mut buf = vec![0u8; len];
                let opts = FileSystemReadWriteOptions::new();
                opts.set_at(off.0 as f64);
                if h.read_with_u8_array_and_options(&mut buf, &opts).is_err() {
                    return None;
                }

                let res = if ts {
                    JsValue::from(js_sys::Uint8Array::from(&buf[..]))
                } else {
                    let text = d.decode_with_u8_array(&buf).unwrap_or_default();
                    let sep = match mode {
                        LineEnding::CR => "\r",
                        LineEnding::NLCR => "\r\n",
                        _ => "\n",
                    };
                    let out = text
                        .split(sep)
                        .map(|l| if l.len() > 15 { &l[15..] } else { l })
                        .collect::<Vec<_>>()
                        .join(sep);
                    JsValue::from(e.encode_with_input(&out))
                };
                Some((Ok(res), ByteOffset(off.0 + len as u64)))
            }
        });
        Ok(ReadableStream::from_stream(stream).into_raw().into())
    }

    // --- Private Log Logic ---
    fn write_and_update_internal(
        &mut self,
        text: &str,
        offsets: Vec<ByteOffset>,
        filtered: Vec<LineRange>,
    ) -> Result<(), LogError> {
        let start = self.storage.backend.get_file_size()?;
        self.storage
            .backend
            .write_at(start, self.storage.encoder.encode_with_input(text).as_ref())?;
        for off in offsets {
            self.index.push_line(start + off.0);
        }
        for mut r in filtered {
            r.start = start + r.start.0;
            r.end = start + r.end.0;
            self.index.push_filtered(r);
        }
        Ok(())
    }

    fn prepare_batch_with_formatter(
        &mut self,
        chunk: &str,
        formatter: &dyn LogFormatterStrategy,
    ) -> (String, Vec<ByteOffset>, Vec<LineRange>) {
        let max_len = formatter.max_line_length();

        // 1. If leftover is already too long, force a split before even adding new chunk
        if !self.leftover_chunk.is_empty() && self.leftover_chunk.len() >= max_len {
            self.leftover_chunk.push('\n');
        }

        let full_text = if self.leftover_chunk.is_empty() {
            Cow::Borrowed(chunk)
        } else {
            Cow::Owned(format!("{}{}", self.leftover_chunk, chunk))
        };

        let mut raw_lines: Vec<&str> = match self.formatter.line_ending_mode {
            LineEnding::None | LineEnding::NL => full_text.split('\n'),
            LineEnding::CR => full_text.split('\x0D'),
            LineEnding::NLCR => full_text.split('\n'),
        }
        .collect();

        // The last part is the new leftover
        self.leftover_chunk = raw_lines.pop().unwrap_or("").to_string();

        let mut batch = String::with_capacity(full_text.len() * 2);
        let mut offsets = Vec::with_capacity(raw_lines.len());
        let mut filtered = Vec::new();
        let mut relative_offset = ByteOffset(0);
        let timestamp = self.formatter.get_timestamp();

        for line in raw_lines {
            let cleaned = formatter.clean_line_ending(line);

            // 2. Sub-split if this line itself is too long
            let mut start = 0;
            while start < cleaned.len() {
                let end = (start + max_len).min(cleaned.len());
                let sub_line = &cleaned[start..end];

                let start_pos = batch.len();
                let formatted = formatter.format(sub_line, &timestamp);
                batch.push_str(&formatted);
                let line_len = (batch.len() - start_pos) as u64;

                if self.index.is_filtering
                    && self
                        .index
                        .active_filter
                        .as_ref()
                        .map_or(false, |f| f.matches(&batch[start_pos..]))
                {
                    filtered.push(LineRange {
                        start: relative_offset,
                        end: relative_offset + line_len,
                    });
                }

                relative_offset = relative_offset + line_len;
                offsets.push(relative_offset);
                start = end;
            }
        }

        (batch, offsets, filtered)
    }

    fn decode_with_streaming_internal(&self, chunk: &[u8]) -> Result<String, JsValue> {
        let opts = web_sys::TextDecodeOptions::new();
        opts.set_stream(true);
        self.storage
            .decoder
            .decode_with_u8_array_and_options(chunk, &opts)
    }

    fn reset_filter_internal(&mut self) -> Result<u32, LogError> {
        self.index.clear_filter();
        Ok(self.index.line_count as u32)
    }
}
