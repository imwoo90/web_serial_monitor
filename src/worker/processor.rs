use crate::state::LineEnding;
use crate::worker::chunk_handler::StreamingLineProcessor;
use crate::worker::error::LogError;

use crate::worker::formatter::{
    DefaultFormatter, HexFormatter, LogFormatter, LogFormatterStrategy,
};
use crate::worker::index::{ByteOffset, LineRange};
use crate::worker::repository::LogRepository;
use crate::worker::storage::StorageBackend;

use wasm_bindgen::prelude::*;
use web_sys::FileSystemSyncAccessHandle;

use crate::config::{MAX_LINE_BYTES, READ_BUFFER_SIZE};

#[wasm_bindgen]
pub struct LogProcessor {
    pub(crate) repository: LogRepository,
    pub(crate) formatter: LogFormatter,
    chunk_handler: StreamingLineProcessor,
}

#[wasm_bindgen]
impl LogProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<LogProcessor, JsValue> {
        LogProcessor::new_internal().map_err(JsValue::from)
    }

    fn new_internal() -> Result<Self, LogError> {
        Ok(LogProcessor {
            repository: LogRepository::new()?,
            formatter: LogFormatter::new(LineEnding::NL),
            chunk_handler: StreamingLineProcessor::new(),
        })
    }

    // --- Public API ---
    pub fn get_line_count(&self) -> u32 {
        self.repository.get_line_count() as u32
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
        self.repository.storage.backend.handle = Some(handle);
        let size = self.repository.storage.backend.get_file_size()?;
        if size.0 > 0 {
            self.repository.reset_index();
            let (mut off, mut buf) = (ByteOffset(0), vec![0u8; READ_BUFFER_SIZE]);
            while off.0 < size.0 {
                let len = (size.0 - off.0).min(buf.len() as u64) as usize;
                self.repository
                    .storage
                    .backend
                    .read_at(off, &mut buf[..len])?;
                for (i, &b) in buf[..len].iter().enumerate() {
                    if b == 10 {
                        self.repository.index.push_line(off + (i as u64 + 1));
                    }
                }
                off = off + (len as u64);
            }
        }
        Ok(())
    }

    pub fn append_chunk(&mut self, chunk: &[u8], is_hex: bool) -> Result<u32, JsValue> {
        self.append_chunk_internal(chunk, is_hex)
            .map_err(JsValue::from)
    }

    fn append_chunk_internal(&mut self, chunk: &[u8], is_hex: bool) -> Result<u32, LogError> {
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
            self.decode_with_streaming(chunk)?
        };

        let timestamp = self.formatter.get_timestamp();
        let is_filtering = self.repository.index.is_filtering;
        let active_filter = self.repository.index.active_filter.clone();

        let (batch, offsets, filtered) = self.chunk_handler.process_chunk(
            &text,
            &*formatter,
            &timestamp,
            is_filtering,
            |text: &str| is_filtering && active_filter.as_ref().is_some_and(|f| f.matches(text)),
        );

        if !batch.is_empty() {
            self.repository.append_lines(&batch, offsets, filtered)?;
        }
        Ok(self.get_line_count())
    }

    pub fn append_log(&mut self, text: String) -> Result<u32, JsValue> {
        self.append_log_internal(text).map_err(JsValue::from)
    }

    fn append_log_internal(&mut self, text: String) -> Result<u32, LogError> {
        let log = format!("[TX] {} {}\n", self.formatter.get_timestamp(), text);
        let len = ByteOffset(log.len() as u64);
        let filtered = if self.repository.index.is_filtering
            && self
                .repository
                .index
                .active_filter
                .as_ref()
                .is_some_and(|f| f.matches(&log))
        {
            vec![LineRange {
                start: ByteOffset(0),
                end: len,
            }]
        } else {
            vec![]
        };
        self.repository.append_lines(&log, vec![len], filtered)?;
        Ok(self.get_line_count())
    }

    pub fn clear(&mut self) -> Result<(), JsValue> {
        self.clear_internal().map_err(JsValue::from)
    }

    fn clear_internal(&mut self) -> Result<(), LogError> {
        self.repository.clear()?;
        self.chunk_handler.clear();
        Ok(())
    }

    fn decode_with_streaming(&self, chunk: &[u8]) -> Result<String, LogError> {
        let opts = web_sys::TextDecodeOptions::new();
        opts.set_stream(true);
        self.repository
            .storage
            .decoder
            .decode_with_u8_array_and_options(chunk, &opts)
            .map_err(LogError::from)
    }
}
