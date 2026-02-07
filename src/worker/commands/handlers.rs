use crate::worker::commands::command::WorkerCommand;
use crate::worker::error::LogError;
use crate::worker::export::LogExporter;
use crate::worker::repository::index::LineIndex;
use crate::worker::repository::storage::StorageBackend;
use crate::worker::search::LogSearcher;
use crate::worker::state::WorkerState;
use crate::worker::types::WorkerMsg;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::JsValue;
use wasm_bindgen_futures::spawn_local;

pub struct NewSessionCommand;

impl WorkerCommand for NewSessionCommand {
    fn execute(
        &self,
        _state: &mut WorkerState,
        _state_rc: &Rc<RefCell<WorkerState>>,
    ) -> Result<bool, JsValue> {
        // Needs async handling by caller
        Ok(false)
    }
}

pub struct AppendChunkCommand {
    pub chunk: Vec<u8>,
    pub is_hex: bool,
}

impl WorkerCommand for AppendChunkCommand {
    fn execute(
        &self,
        state: &mut WorkerState,
        _state_rc: &Rc<RefCell<WorkerState>>,
    ) -> Result<bool, JsValue> {
        let active_line = state.proc.append_chunk(&self.chunk, self.is_hex)?;
        if let Some(line) = active_line {
            state.send_msg(WorkerMsg::ActiveLine(Some(line)));
        } else {
            // Send None to clear if active line became empty (e.g. newline received)
            state.send_msg(WorkerMsg::ActiveLine(None));
        }
        Ok(true)
    }
}

pub struct SetTimestampStateCommand(pub bool);

impl WorkerCommand for SetTimestampStateCommand {
    fn execute(
        &self,
        state: &mut WorkerState,
        _state_rc: &Rc<RefCell<WorkerState>>,
    ) -> Result<bool, JsValue> {
        state.proc.set_timestamp_state(self.0);
        Ok(true)
    }
}

pub struct RequestWindowCommand {
    pub start_line: usize,
    pub count: usize,
}

impl WorkerCommand for RequestWindowCommand {
    fn execute(
        &self,
        state: &mut WorkerState,
        _state_rc: &Rc<RefCell<WorkerState>>,
    ) -> Result<bool, JsValue> {
        let total = state.proc.get_line_count() as usize;
        let (s, e) = (
            self.start_line.min(total),
            (self.start_line + self.count).min(total),
        );
        let mut lines = Vec::with_capacity(e - s);
        let repo = &state.proc.repository;

        for i in s..e {
            if let Some(range) = repo.get_line_range(LineIndex(i)) {
                let buf = repo.read_line(range).map_err(JsValue::from)?;

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
    fn execute(
        &self,
        state: &mut WorkerState,
        _state_rc: &Rc<RefCell<WorkerState>>,
    ) -> Result<bool, JsValue> {
        state.proc.clear()?;
        state.send_msg(WorkerMsg::TotalLines(0));
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
    fn execute(
        &self,
        state: &mut WorkerState,
        state_rc: &Rc<RefCell<WorkerState>>,
    ) -> Result<bool, JsValue> {
        let query = self.query.clone();
        let match_case = self.match_case;
        let use_regex = self.use_regex;
        let invert = self.invert;
        let state_rc_clone = state_rc.clone();

        // Cancel previous search by incrementing search_id
        state.current_search_id += 1;

        spawn_local(async move {
            if let Err(e) = LogSearcher::search_async(
                state_rc_clone.clone(),
                query,
                match_case,
                use_regex,
                invert,
            )
            .await
            {
                state_rc_clone.borrow().send_error(JsValue::from(e));
            }
        });

        Ok(true)
    }
}

pub struct ExportLogsCommand;

impl WorkerCommand for ExportLogsCommand {
    fn execute(
        &self,
        state: &mut WorkerState,
        _state_rc: &Rc<RefCell<WorkerState>>,
    ) -> Result<bool, JsValue> {
        let repo = &state.proc.repository;
        let size = repo.storage.backend.get_file_size()?;
        let handle = repo
            .storage
            .backend
            .handle
            .as_ref()
            .cloned()
            .ok_or_else(|| LogError::Storage("OPFS handle missing for export".into()))
            .map_err(JsValue::from)?;

        let stream = LogExporter::export_logs(handle, size).map_err(JsValue::from)?;

        let resp = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&resp, &"type".into(), &"EXPORT_STREAM".into());
        let _ = js_sys::Reflect::set(&resp, &"stream".into(), &stream);
        let _ = state
            .scope
            .post_message_with_transfer(&resp, &js_sys::Array::of1(&stream));
        Ok(true)
    }
}
