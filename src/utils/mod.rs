pub mod filter;
pub mod format;
pub mod highlight;
pub mod history;
pub mod parser;
pub mod scroll;

pub use filter::LogFilter;
pub use format::format_hex;
pub use highlight::process_log_segments;
pub use history::CommandHistory;
pub use parser::LineParser;
pub use scroll::{calculate_start_index, calculate_window_size};
