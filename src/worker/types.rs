use serde::{Deserialize, Serialize};

/// Message protocol for communicating with Web Worker
/// Note: Do NOT use #[serde(tag = "...")] or #[serde(rename = "...")]
/// as gloo-worker's default Bincode codec does not support them.
#[derive(Serialize, Deserialize, Debug)]
pub enum WorkerMsg {
    TotalLines(usize),
    LogWindow {
        start_line: usize,
        lines: Vec<String>,
    },
    AppendLog(String),
    RequestWindow {
        start_line: usize,
        count: usize,
    },
    ExportLogs {
        include_timestamp: bool,
    },
    Clear,
    Error(String),
    SearchLogs {
        query: String,
        match_case: bool,
        use_regex: bool,
        invert: bool,
    },
    SetLineEnding(String),
    NewSession,
    AppendChunk {
        chunk: Vec<u8>,
        is_hex: bool,
    },
}
