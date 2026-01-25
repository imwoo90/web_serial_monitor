use crate::worker::state::WorkerState;
use wasm_bindgen::prelude::JsValue;

/// Command interface for worker operations
pub trait WorkerCommand {
    /// Executes the command on the worker state
    /// Returns Ok(true) if synchronous, Ok(false) if asynchronous handling is needed
    fn execute(&self, state: &mut WorkerState) -> Result<bool, JsValue>;
}
