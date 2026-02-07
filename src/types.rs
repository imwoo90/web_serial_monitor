use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Highlight {
    pub id: usize,
    pub text: String,
    pub color: &'static str,
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum LineEnding {
    #[default]
    None,
    NL,
    CR,
    NLCR,
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum ViewMode {
    #[default]
    Monitoring,
    Terminal,
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum Parity {
    #[default]
    None,
    Even,
    Odd,
}

impl fmt::Display for Parity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Parity::None => write!(f, "none"),
            Parity::Even => write!(f, "even"),
            Parity::Odd => write!(f, "odd"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum FlowControl {
    #[default]
    None,
    Hardware,
}

impl fmt::Display for FlowControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlowControl::None => write!(f, "none"),
            FlowControl::Hardware => write!(f, "hardware"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum WorkerMsg {
    NewSession,
    AppendChunk {
        chunk: Vec<u8>,
        is_hex: bool,
    },
    SetTimestampState(bool),

    RequestWindow {
        start_line: usize,
        count: usize,
    },
    LogWindow {
        start_line: usize,
        lines: Vec<(usize, String)>,
    },
    TotalLines(usize),
    Clear,
    SearchLogs {
        query: String,
        match_case: bool,
        use_regex: bool,
        invert: bool,
    },
    ExportLogs {
        include_timestamp: bool,
    },
    ActiveLine(Option<String>),
    SetMode(ViewMode),
    Error(String),
}
