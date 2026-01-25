use crate::state::LineEnding;
use crate::worker::chunk_handler::StreamingLineProcessor;
use crate::worker::error::LogError;

use crate::worker::formatter::LogFormatter;
use crate::worker::index::{ByteOffset, LineRange};
use crate::worker::repository::LogRepository;

use wasm_bindgen::prelude::*;
use web_sys::FileSystemSyncAccessHandle;

use crate::config::MAX_LINE_BYTES;

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
        self.repository.initialize_storage(handle)
    }

    pub fn append_chunk(&mut self, chunk: &[u8], is_hex: bool) -> Result<u32, JsValue> {
        self.append_chunk_internal(chunk, is_hex)
            .map_err(JsValue::from)
    }

    fn append_chunk_internal(&mut self, chunk: &[u8], is_hex: bool) -> Result<u32, LogError> {
        let formatter = self.formatter.create_strategy(is_hex, MAX_LINE_BYTES);

        let text = if is_hex {
            formatter.format_chunk(chunk)
        } else {
            self.decode_with_streaming(chunk)?
        };

        let timestamp = self.formatter.get_timestamp();
        let repo = &self.repository;
        let is_filtering = repo.is_filtering();

        let (batch, offsets, filtered) = self.chunk_handler.process_chunk(
            &text,
            &*formatter,
            &timestamp,
            is_filtering,
            |text: &str| repo.matches_active_filter(text),
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
        let filtered = if self.repository.matches_active_filter(&log) {
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
