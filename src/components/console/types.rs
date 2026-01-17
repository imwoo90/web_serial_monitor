use serde::{Deserialize, Serialize};

/// Web Worker와 통신하기 위한 메시지 프로토콜
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum WorkerMsg {
    #[serde(rename = "INITIALIZED")]
    Initialized(String),
    #[serde(rename = "TOTAL_LINES")]
    TotalLines(usize),
    #[serde(rename = "LOG_WINDOW")]
    LogWindow {
        #[serde(rename = "startLine")]
        start_line: usize,
        lines: Vec<String>,
    },
    #[serde(rename = "APPEND_LOG")]
    AppendLog(String),
    #[serde(rename = "REQUEST_WINDOW")]
    RequestWindow {
        #[serde(rename = "startLine")]
        start_line: usize,
        count: usize,
    },
}

pub const LINE_HEIGHT: f64 = 20.0;
pub const HEADER_OFFSET: f64 = 150.0;
pub const TOP_BUFFER: usize = 10;
pub const BOTTOM_BUFFER_EXTRA: usize = 40;
