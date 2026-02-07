use crate::worker::chunk_handler::StreamingLineProcessor;
use crate::worker::error::LogError;

use crate::worker::formatter::LogFormatter;

use crate::worker::repository::LogRepository;

use wasm_bindgen::prelude::*;
use web_sys::FileSystemSyncAccessHandle;

use crate::config::MAX_LINE_BYTES;

#[wasm_bindgen]
pub struct LogProcessor {
    pub(crate) repository: LogRepository,
    pub(crate) formatter: LogFormatter,
    pub(crate) show_timestamps: bool,
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
            formatter: LogFormatter::new(),
            show_timestamps: false,
            chunk_handler: StreamingLineProcessor::new(),
        })
    }

    // --- Public API ---
    pub fn get_line_count(&self) -> u32 {
        self.repository.get_line_count() as u32
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

    pub fn append_chunk(&mut self, chunk: &[u8], is_hex: bool) -> Result<Option<String>, JsValue> {
        self.append_chunk_internal(chunk, is_hex)
            .map_err(JsValue::from)
    }

    fn append_chunk_internal(
        &mut self,
        chunk: &[u8],
        is_hex: bool,
    ) -> Result<Option<String>, LogError> {
        let formatter = self.formatter.create_strategy(is_hex, MAX_LINE_BYTES);
        let timestamp = if self.show_timestamps {
            self.formatter.get_timestamp()
        } else {
            String::new()
        };

        let repo = &self.repository;
        let is_filtering = repo.is_filtering();
        let filter_matcher = |text: &str| repo.matches_active_filter(text);

        let (batch, offsets, filtered, active_line) = if is_hex {
            let text = formatter.format_chunk(chunk);
            let (b, o, f) = self.chunk_handler.process_hex_lines(
                &text,
                &*formatter,
                &timestamp,
                is_filtering,
                filter_matcher,
            );
            (b, o, f, None)
        } else {
            self.chunk_handler.process_vt100(
                chunk,
                &*formatter,
                &timestamp,
                is_filtering,
                filter_matcher,
            )
        };

        if !batch.is_empty() {
            self.repository.append_lines(&batch, offsets, filtered)?;
        }
        Ok(active_line)
    }

    pub fn set_timestamp_state(&mut self, enabled: bool) {
        self.show_timestamps = enabled;
    }

    pub fn clear(&mut self) -> Result<(), JsValue> {
        self.clear_internal().map_err(JsValue::from)
    }

    fn clear_internal(&mut self) -> Result<(), LogError> {
        self.repository.clear()?;
        self.chunk_handler.clear();
        Ok(())
    }
}
