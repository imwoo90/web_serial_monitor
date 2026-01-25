pub mod chunk_handler;
pub mod commands;
pub mod dispatcher;
pub mod error;
pub mod export;
pub mod formatter;
pub mod index;
pub mod lifecycle;
pub mod processor;
pub mod repository;
pub mod search;
pub mod state;
pub mod storage;
pub mod types;

// Re-export public functions
pub use lifecycle::get_app_script_path;
