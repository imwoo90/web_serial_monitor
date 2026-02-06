pub mod ansi_decoder;
pub mod file_save;
pub mod format;
pub mod history;
pub mod macros;
pub mod scroll;
pub mod serial_api;
pub mod simulation;

pub use ansi_decoder::decode_ansi_text;
pub use format::{format_hex_input, parse_hex_string, send_chunk_to_worker, send_worker_msg};
pub use history::CommandHistory;
pub use macros::MacroStorage;
pub use scroll::{calculate_start_index, calculate_window_size};
pub use serial_api as serial;
