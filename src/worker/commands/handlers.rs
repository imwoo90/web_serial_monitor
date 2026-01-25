use crate::worker::commands::command::WorkerCommand;
use crate::worker::error::LogError;
use crate::worker::export::LogExporter;
use crate::worker::index::LineIndex;
use crate::worker::search::LogSearcher;
use crate::worker::state::WorkerState;
use crate::worker::storage::StorageBackend;
use crate::worker::types::WorkerMsg;
use wasm_bindgen::prelude::JsValue;

pub struct NewSessionCommand;

impl WorkerCommand for NewSessionCommand {
    fn execute(&self, _state: &mut WorkerState) -> Result<bool, JsValue> {
        // Needs async handling by caller
        Ok(false)
    }
}

pub struct AppendChunkCommand {
    pub chunk: Vec<u8>,
    pub is_hex: bool,
}

impl WorkerCommand for AppendChunkCommand {
    fn execute(&self, state: &mut WorkerState) -> Result<bool, JsValue> {
        state.proc.append_chunk(&self.chunk, self.is_hex)?;
        Ok(true)
    }
}

pub struct AppendLogCommand {
    pub text: String,
}

impl WorkerCommand for AppendLogCommand {
    fn execute(&self, state: &mut WorkerState) -> Result<bool, JsValue> {
        state.proc.append_log(self.text.clone())?;
        Ok(true)
    }
}

pub struct RequestWindowCommand {
    pub start_line: usize,
    pub count: usize,
}

impl WorkerCommand for RequestWindowCommand {
    fn execute(&self, state: &mut WorkerState) -> Result<bool, JsValue> {
        let total = state.proc.get_line_count() as usize;
        let (s, e) = (
            self.start_line.min(total),
            (self.start_line + self.count).min(total),
        );
        let mut lines = Vec::with_capacity(e - s);
        let repo = &state.proc.repository;

        for i in s..e {
            if let Some(range) = repo.index.get_line_range(LineIndex(i)) {
                let mut buf = vec![0u8; (range.end.0 - range.start.0) as usize];
                repo.storage
                    .backend
                    .read_at(range.start, &mut buf)
                    .map_err(LogError::from)?;

                let text = repo
                    .storage
                    .decoder
                    .decode_with_u8_array(&buf)
                    .map_err(LogError::from)?
                    .trim_end_matches('\n')
                    .to_string();
                lines.push((i, text));
            }
        }

        state.send_msg(WorkerMsg::LogWindow {
            start_line: self.start_line,
            lines,
        });
        Ok(true)
    }
}

pub struct ClearCommand;

impl WorkerCommand for ClearCommand {
    fn execute(&self, state: &mut WorkerState) -> Result<bool, JsValue> {
        state.proc.clear()?;
        state.send_msg(WorkerMsg::TotalLines(0));
        Ok(true)
    }
}

pub struct SetLineEndingCommand {
    pub mode: String,
}

impl WorkerCommand for SetLineEndingCommand {
    fn execute(&self, state: &mut WorkerState) -> Result<bool, JsValue> {
        state.proc.set_line_ending(&self.mode);
        Ok(true)
    }
}

pub struct SearchLogsCommand {
    pub query: String,
    pub match_case: bool,
    pub use_regex: bool,
    pub invert: bool,
}

impl WorkerCommand for SearchLogsCommand {
    fn execute(&self, state: &mut WorkerState) -> Result<bool, JsValue> {
        let repo = &mut state.proc.repository;
        let count = LogSearcher::search(
            &mut repo.storage,
            &mut repo.index,
            self.query.clone(),
            self.match_case,
            self.use_regex,
            self.invert,
        )
        .map_err(JsValue::from)?;

        state.send_msg(WorkerMsg::TotalLines(count as usize));
        Ok(true)
    }
}

pub struct ExportLogsCommand {
    pub include_timestamp: bool,
}

impl WorkerCommand for ExportLogsCommand {
    fn execute(&self, state: &mut WorkerState) -> Result<bool, JsValue> {
        let repo = &state.proc.repository;
        let size = repo
            .storage
            .backend
            .get_file_size()
            .map_err(LogError::from)?;
        let handle = repo
            .storage
            .backend
            .handle
            .as_ref()
            .cloned()
            .ok_or_else(|| LogError::Storage("OPFS handle missing for export".into()))
            .map_err(JsValue::from)?;

        let stream = LogExporter::export_logs(
            handle,
            repo.storage.decoder.clone(),
            repo.storage.encoder.clone(),
            state.proc.formatter.line_ending_mode,
            size,
            self.include_timestamp,
        )
        .map_err(JsValue::from)?;

        let resp = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&resp, &"type".into(), &"EXPORT_STREAM".into());
        let _ = js_sys::Reflect::set(&resp, &"stream".into(), &stream);
        let _ = state
            .scope
            .post_message_with_transfer(&resp, &js_sys::Array::of1(&stream));
        Ok(true)
    }
}
