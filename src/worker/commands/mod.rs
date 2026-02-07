pub mod command;
pub mod handlers;

pub use command::WorkerCommand;
pub use handlers::*;

use crate::worker::types::WorkerMsg;

/// Factory to convert WorkerMsg into a specific Command
pub fn create_command_from_msg(msg: WorkerMsg) -> Box<dyn WorkerCommand> {
    match msg {
        WorkerMsg::NewSession => Box::new(NewSessionCommand),
        WorkerMsg::AppendChunk { chunk, is_hex } => Box::new(AppendChunkCommand { chunk, is_hex }),
        WorkerMsg::SetTimestampState(enabled) => Box::new(SetTimestampStateCommand(enabled)),

        WorkerMsg::RequestWindow { start_line, count } => {
            Box::new(RequestWindowCommand { start_line, count })
        }
        WorkerMsg::Clear => Box::new(ClearCommand),

        WorkerMsg::SearchLogs {
            query,
            match_case,
            use_regex,
            invert,
        } => Box::new(SearchLogsCommand {
            query,
            match_case,
            use_regex,
            invert,
        }),
        WorkerMsg::ExportLogs { .. } => Box::new(ExportLogsCommand),

        _ => Box::new(NoOpCommand), // Fallback for handled/error messages
    }
}

pub struct NoOpCommand;
impl WorkerCommand for NoOpCommand {
    fn execute(
        &self,
        _state: &mut crate::worker::state::WorkerState,
        _state_rc: &std::rc::Rc<std::cell::RefCell<crate::worker::state::WorkerState>>,
    ) -> Result<bool, wasm_bindgen::prelude::JsValue> {
        Ok(true)
    }
}
