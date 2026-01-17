pub mod file_save;
// pub mod filter; // Removed
pub mod format;
pub mod highlight;
pub mod history;
pub mod macros;
// pub mod parser; // Moved to Worker
pub mod scroll;

pub use format::{format_hex_input, parse_hex_string, send_chunk_to_worker}; // Removed format_hex
pub use highlight::process_log_segments;
pub use history::CommandHistory;
pub use macros::MacroStorage;
// pub use parser::LineParser;
pub use scroll::{calculate_start_index, calculate_window_size};
